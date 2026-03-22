// Copyright 2026 synning360
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

pub struct Config {
    pub map_path: String,
    pub verbosity: i32,
    pub map_args: Vec<String>,
}

pub fn preprocess() -> Config {
    let raw_args: Vec<String> = std::env::args().collect();
    
    let mut map_path = String::from("build.map");
    let mut verbosity = 1; 
    let mut map_args = Vec::new();

    let mut i = 1;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "-v" => {
                if verbosity == 0 {
                    eprintln!("CFBS Error:   Cannot have both -q and -v.");
                    println!("Suggestion:   Try removing -v, or -q.");
                    std::process::exit(1);
                }

                verbosity = 2;
                i += 1;
            }
            "-q" => {
                if verbosity == 2 {
                    eprintln!("CFBS Error:   Cannot have both -q and -v.");
                    println!("Suggestion:   Try removing -q, or -v.");
                    std::process::exit(1);
                }

                verbosity = 0;
                i += 1;
            }
            "-dir" => {
                if map_path != "build.map" {
                    eprintln!("CFBS Error:   Cannot have multiple -dir flags.");
                    println!("Suggestion:   Try removing this -dir flag, keeping the first one.");
                    std::process::exit(1);
                }

                if i + 1 < raw_args.len() {
                    map_path = raw_args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("CFBS Error:  -dir requires a path to a file path.");
                    println!("Suggestion:   Try adding a path to a build script.");
                    std::process::exit(1);
                }
            }
            _ => {
                map_args.push(raw_args[i].clone());
                i += 1;
            }
        }
    }

    Config {
        map_path,
        verbosity,
        map_args,
    }
}