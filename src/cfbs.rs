// Copyright 2026 synning360
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use std::fs;

mod utils;
use utils::preprocess;
use utils::toolchain;
use utils::execute;

fn main() {
    let config = preprocess::preprocess();

    if config.verbosity >= 1 { println!("--- CFBS v1.0.0-Alpha ---"); }

    let content = match fs::read_to_string(&config.map_path) {
        Ok(c) => {
            if config.verbosity == 2 { 
                println!("Success: Loaded script from {}", config.map_path); 
            }
            c 
        }
        Err(_) => {
            eprintln!("CFBS Error:      Could not read '{}'", config.map_path);
            std::process::exit(1);
        }
    };

    let toolchain = toolchain::Toolchain::new();
    
    if config.verbosity == 2 {
        println!("Found Linker:  {}", toolchain.rust_ld);
        println!("Found Objcopy: {}", toolchain.rust_obj);
    }

    let mut executor = execute::Executor::new(&toolchain, config.map_args, config.verbosity);
    executor.interpret(content);
}