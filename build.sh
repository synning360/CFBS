#!/bin/sh
set -e

# Copyright 2026 synning360
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.

echo "--- Building CFBS v1.0.0-Alpha ---"
rustc src/cfbs.rs -o cfbs

printf "Install to /usr/local/bin? (y/n): "
read -r choice

case "$choice" in 
  y|Y ) 
    echo "Installing..."
    
    sudo mkdir -p /usr/local/bin
    sudo cp cfbs /usr/local/bin/cfbs
    sudo chmod +x /usr/local/bin/cfbs

    echo "Done!"
    ;;
  * ) 
    echo "Quitting..."
    ;;
esac