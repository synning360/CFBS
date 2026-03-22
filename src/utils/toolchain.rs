// Copyright 2026 synning360
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use std::process::Command;
use std::path::Path;
use std::fs;

pub struct Toolchain {
    pub rust_ld: String,
    pub rust_obj: String,
    pub cc: String,
    pub as_cmd: String,
}

impl Toolchain {
    pub fn new() -> Self {
        let sysroot = get_sysroot();
        
        let lld_path = find_binary(Path::new(&sysroot), "rust-lld")
            .unwrap_or_else(|| "rust-lld".to_string());
            
        let obj_path = find_binary(Path::new(&sysroot), "llvm-objcopy")
            .unwrap_or_else(|| "llvm-objcopy".to_string());

        Self {
            rust_ld: format!("{} -flavor gnu", lld_path),
            rust_obj: obj_path,
            cc: "rustc".to_string(),
            as_cmd: "clang".to_string(),
        }
    }
}

fn get_sysroot() -> String {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .expect("CFBS Error:    rustc not found in PATH\nSuggestion:    Do you have rustc installed?");
    
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn find_binary(path: &Path, name: &str) -> Option<String> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(found) = find_binary(&p, name) {
                    return Some(found);
                }
            } else if let Some(file_name) = p.file_name() {
                if file_name == name || file_name == format!("{}.exe", name).as_str() {
                    return Some(p.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}