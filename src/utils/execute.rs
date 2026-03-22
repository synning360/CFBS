// Copyright 2026 synning360
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use std::collections::HashMap;
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use crate::utils::toolchain::Toolchain;

pub struct Executor {
    pub vars: HashMap<String, String>,
    pub actions: HashMap<String, Vec<String>>, 
    pub skip_stack: Vec<bool>,
    pub verbosity: i32,
    active_group: Option<String>,
    group_overrides: HashMap<String, String>,
}

impl Executor {
    pub fn new(tc: &Toolchain, args: Vec<String>, verbosity: i32) -> Self {
        let mut vars = HashMap::new();
        vars.insert("as".into(), tc.as_cmd.clone());
        vars.insert("cc".into(), tc.cc.clone());
        vars.insert("ld".into(), tc.rust_ld.clone());
        vars.insert("obj".into(), tc.rust_obj.clone());
        vars.insert("target".into(), "host".into());
        vars.insert("flags".into(), "".into());
        vars.insert("src".into(), "src".into());
        vars.insert("out".into(), "out".into());

        for (i, val) in args.iter().enumerate() {
            vars.insert(format!("arg{}", i), val.clone());
        }

        Self { 
            vars, 
            actions: HashMap::new(),
            skip_stack: Vec::new(), 
            verbosity, 
            active_group: None,
            group_overrides: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, content: String) {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut i = 0;

        while i < lines.len() {
            let line_num = i + 1;
            let raw_line = &lines[i];
            let clean = raw_line.split(';').next().unwrap_or("").trim();
            
            if clean.is_empty() { i += 1; continue; }

            let tokens: Vec<String> = clean.split_whitespace().map(|s| s.to_string()).collect();
            let is_skipping = self.skip_stack.iter().any(|&s| s);
            let keyword = tokens[0].to_lowercase();

            match keyword.as_str() {
                "action" => {
                    let name = tokens.get(1).cloned().unwrap_or_default();
                    
                    let mut action_body = Vec::new();
                    let mut depth = 1;
                    let start_index = i;
                    i += 1;
                    
                    while i < lines.len() && depth > 0 {
                        let sub_line = lines[i].split(';').next().unwrap_or("").trim();
                        let sub_tokens: Vec<&str> = sub_line.split_whitespace().collect();
                        
                        if !sub_tokens.is_empty() {
                            let sub_keyword = sub_tokens[0].to_lowercase();
                            if ["action", "group", "if"].contains(&sub_keyword.as_str()) { depth += 1; }
                            if sub_keyword == "end" { depth -= 1; }
                        }
                        
                        if depth > 0 { action_body.push(lines[i].clone()); }
                        i += 1;
                    }
                    
                    self.actions.insert(name.clone(), action_body);

                    let is_cli_call = self.vars.get("arg0") == Some(&name);
                    if is_cli_call && !is_skipping {
                        i = start_index + 1;
                        self.skip_stack.push(false);
                    } else {
                        continue; 
                    }
                }
                "group" => {
                    let name = tokens.get(1).cloned().unwrap_or_default();
                    self.active_group = Some(name);
                    self.group_overrides = self.parse_with_tokens(&tokens);
                    self.skip_stack.push(is_skipping);
                }
                "if" => {
                    let result = self.eval_full_logic(&tokens[1..]);
                    self.skip_stack.push(!result || is_skipping);
                }
                "print" if !is_skipping => {
                    self.handle_print(&tokens);
                }
                "end" => {
                    if self.skip_stack.pop().is_none() { self.error(line_num, "Unexpected 'end'."); }
                    if self.skip_stack.is_empty() { 
                        self.active_group = None; 
                        self.group_overrides.clear();
                    }
                }
                _ if tokens.len() >= 3 && tokens[1].to_lowercase() == "is" => {
                    if !is_skipping { self.handle_assignment(&tokens); }
                }
                "in" if !is_skipping => self.run_in_recursive(&tokens, line_num),
                "and" if !is_skipping => self.compile_file(&tokens[1], &tokens, line_num),
                "link" if !is_skipping => self.run_link(&tokens, line_num),
                "obj" if !is_skipping => self.run_objcopy(&tokens, line_num),
                "move" if !is_skipping => self.run_move(&tokens, line_num),
                "delete" if !is_skipping => self.run_delete(&tokens, line_num),

                _ if !is_skipping => {
                    let func_name = &tokens[0];
                    if let Some(body) = self.actions.get(func_name).cloned() {
                        if self.verbosity == 2 { println!("  [Action] calling '{}'", func_name); }
                        self.interpret(body.join("\n"));
                    } else {
                        self.dispatch(clean, line_num, "External");
                    }
                }
                _ => {} 
            }
            i += 1;
        }
    }

    fn handle_print(&self, tokens: &[String]) {
    let mut output = Vec::new();
    for token in &tokens[1..] {
        if token.contains('"') {
            let stripped = token.replace("\"", "");
            if !stripped.is_empty() {
                output.push(stripped);
            }
        } else {
            output.push(self.vars.get(token).cloned().unwrap_or(token.clone()));
        }
    }
    println!("{}", output.join(" "));
}

    fn handle_assignment(&mut self, tokens: &[String]) {
        if tokens[2].to_lowercase() == "read" {
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            self.vars.insert(tokens[0].clone(), input.trim().to_string());
        } else {
            let val = tokens[2..].join(" ");
            self.vars.insert(tokens[0].clone(), val.replace("dir ", ""));
        }
    }

    fn eval_full_logic(&self, tokens: &[String]) -> bool {
        let line = tokens.join(" ");
        let or_parts: Vec<&str> = line.split(" or ").collect();
        let mut final_res = false;
        for part in or_parts {
            let and_parts: Vec<&str> = part.split(" and ").collect();
            let mut and_res = true;
            for a_part in and_parts {
                let xor_parts: Vec<&str> = a_part.split(" but not ").collect();
                let mut xor_res = self.eval_single_cond(xor_parts[0]);
                for x_part in xor_parts.iter().skip(1) {
                    xor_res ^= self.eval_single_cond(x_part);
                }
                and_res &= xor_res;
            }
            final_res |= and_res;
        }
        final_res
    }

    fn eval_single_cond(&self, cond: &str) -> bool {
        let t: Vec<&str> = cond.split_whitespace().collect();
        if t.len() < 3 { return false; }
        let left = self.vars.get(t[0]).cloned().unwrap_or(t[0].to_string());
        let right = t[2].replace("\"", "");
        if t[1] == "is" { left == right } else { left != right }
    }

    fn run_in_recursive(&self, tokens: &[String], line: usize) {
        let dir_val = &tokens[1];
        let target_dir = self.vars.get(dir_val).cloned().unwrap_or(dir_val.clone());
        let src_root = self.vars.get("src").unwrap();
        let full_path = Path::new(src_root).join(&target_dir);
        let mut stack = vec![full_path];
        while let Some(curr_path) = stack.pop() {
            if let Ok(entries) = fs::read_dir(curr_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() { stack.push(path); }
                    else if path.is_file() {
                        let rel = path.strip_prefix(src_root).unwrap().to_str().unwrap();
                        self.compile_file(rel, tokens, line);
                    }
                }
            }
        }
    }

    fn compile_file(&self, rel_path: &str, tokens: &[String], line: usize) {
        let local_overrides = self.parse_with_tokens(tokens);
        let get_var = |k: &str| {
            local_overrides.get(k)
                .or(self.group_overrides.get(k))
                .cloned()
                .unwrap_or_else(|| self.vars.get(k).cloned().unwrap_or_default())
        };
        let cc = get_var("cc");
        let flags = get_var("flags");
        let src_root = self.vars.get("src").unwrap();
        let out_root = self.vars.get("out").unwrap();
        let group = self.active_group.as_ref().expect("Instruction outside group");
        let mut obj_path = PathBuf::from(out_root);
        obj_path.push(group);
        obj_path.push(rel_path);
        obj_path.set_extension("o");
        if let Some(parent) = obj_path.parent() { fs::create_dir_all(parent).ok(); }
        let cmd = format!("{} {} -c {} -o {}", cc, flags, Path::new(src_root).join(rel_path).display(), obj_path.display());
        self.dispatch(&cmd, line, "Compile");
    }

    fn run_link(&self, tokens: &[String], line: usize) {
        let group = &tokens[1];
        let out_root = self.vars.get("out").unwrap();
        let group_path = Path::new(out_root).join(group);
        let mut objects = Vec::new();
        let mut stack = vec![group_path];
        while let Some(path) = stack.pop() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); }
                    else if p.extension().map_or(false, |ext| ext == "o") {
                        objects.push(format!("'{}'", p.to_str().unwrap()));
                    }
                }
            }
        }
        let ld = self.vars.get("lf").unwrap();
        let script = if tokens.contains(&"as".to_string()) { tokens[3].clone() } else { format!("link/{}.ld", group) };
        let cmd = format!("{} -T {} {} -o {}/{}.elf", ld, script, objects.join(" "), out_root, group);
        self.dispatch(&cmd, line, "Link");
    }

    fn run_objcopy(&self, tokens: &[String], line: usize) {
        let group = &tokens[1];
        let obj = self.vars.get("obj").unwrap();
        let out = self.vars.get("out").unwrap();
        let ext = if tokens.contains(&"as".to_string()) { &tokens[3] } else { "bin" };
        let cmd = format!("{} -O binary {}/{}.elf {}/{}.{}", obj, out, group, out, group, ext);
        self.dispatch(&cmd, line, "objcopy");
    }

    fn run_move(&self, tokens: &[String], line: usize) {
        let group = &tokens[1];
        let destination = &tokens[3];
        let out = self.vars.get("out").unwrap();
        let target_path = if destination == "main" { "." } else { destination };
        let cmd = format!("mv {}/{}* {}", out, group, target_path);
        self.dispatch(&cmd, line, "move");
    }

    fn run_delete(&self, tokens: &[String], line: usize) {
        let target = &tokens[1];
        let cmd = format!("rm -rf {}", target);
        self.dispatch(&cmd, line, "delete");
    }

    fn parse_with_tokens(&self, tokens: &[String]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(pos) = tokens.iter().position(|t| t == "with") {
            let mut i = pos + 1;
            while i < tokens.len() {
                let key = &tokens[i];
                if i + 1 < tokens.len() {
                    map.insert(key.clone(), tokens[i+1].clone());
                    i += 2;
                } else {
                    let val = self.vars.get(key).cloned().unwrap_or(key.clone());
                    map.insert("flags".into(), val);
                    break;
                }
            }
        }
        map
    }

    fn dispatch(&self, cmd: &str, line: usize, label: &str) {
        if self.verbosity == 2 { println!("  [{}] line {}", label, line); }
        if self.verbosity == 2 { println!("    CMD: {}", cmd); }
        let status = Command::new("sh").arg("-c").arg(cmd).status();
        if !status.map(|s| s.success()).unwrap_or(false) { self.error(line, &format!("{} failed.", label)); }
    }

    fn error(&self, line: usize, msg: &str) {
        eprintln!("CFBS Error:      {}", msg);
        println!("Line:             {}", line);
        std::process::exit(1);
    }
}
