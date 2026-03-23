# Carbon Fiber Build System
v1.0.0-Alpha, **Copyright (C) 2026, synning360**

# What is CFBS?
CFBS (Carbon Fiber Build System) is a replacement to Make targeted at bare-metal programming. It uses simpler lists of things to compile, groups things together, helping you organize files and automate your build process, and giving you the control Cargo took away without the complexity of Make.

# Why use CFBS over Make and others?
CFBS offers many listed benefits to you
- **1. Syntax**
    <br>Unlike Make's complex syntax with tons of variables and constant directory management and super long lines to get a single thing done, CFBS uses keywords and groups so you worry more about the code you want to build and less about the build script. **And, it doesn't require indents.**

- **2. Groups**
    <br>In Make, you are constantly worried about directories and file paths. Here, you'll never have to worry about them again. Instead of constantly repeating the same path and dealing with path directories, you make a group, and from there on, all of them can be accessed by that group.

- **3. Error Detection**
    <br>If you use Make, you know what I'm talking about. Returning "Syntax Error" without a single line or any details at all. CFBS provides a line and reason, and even a way to fix the syntax.

- **4. Organization**
    <br>In Make, you have to organize everything manually and it makes a mess. Here, it automatically organizes everything for you, providing a clean and navigable output directory instead of a rat's nest.

# CFBS arguments

- **`-q`: Quite mode**
    <br>Quite mode prevents any output except for errors.

- **`-v`: Verbose mode**
    <br>Verbose mode outputs anything possible, used for debugging.

- **`-dir`: Custom build map**
    <br>If you aren't using the name `build.map`, or it's in another directory, this can be used.

# Requirements
This program requires the following:
- **`rustc`**: For compiling it and using it to build Rust

## CFBS `build.map`
CFBS relies on a text file called `build.map` to tell it how to build your project. In it, you define what compilers/assemblers to use, how to group things, and what linker scripts to use for what groups. Here's an example of a `build.map`. 
```txt
; global defines

TARGET is aarch64-unknown-none          ; compile & assemble for aarch64

flags is -O3

; how much to do at once

JOBS is 8                               ; set max jobs to 8

; custom variables

these is --test-flag
core is dir this/kernel                 ; define the kernel directory

; compiling

group kernel                            ; compile the group below together as "kernel"
    in core                             ; everthing in src/kernel
    and boot/thing.c with CC clang -O0  ; also compile this with clang into the group with -O0
    with these                          ; use flags defined in a variable
end

; finishing

link kernel                             ; link kernel group with link/kernel.ld
obj kernel as img                       ; objcopy the raw binary to out/kernel.img
move kernel to main                     ; move project/out/kernel.img to project/kernel.img
```

## Standard file paths
CFBS automatically outputs to `out/` and compiles in `src/`, this can be changed using `out is path/to/it` and `src is path/to/it`. When using `main`, this is a path to `project/`, so you can use it to move from `project/out` to `project/`

## Global definitions
Unlike Makefile, you only need to define this once, and they're globally used unless a `with` overrides them.

| Variable | Meaning |
|-|-|
| AS | Which assembler |
| CC | Which compiler |
| LD | Which linker |
| OBJ | Which objcopy |
| TARGET | Which target |
| FLAGS | Which flags |

### Their defaults
These default to what is listed below.
| Variable | Default |
|-|-|
| AS | clang |
| CC | rustc |
| LD | rustc-ld |
| OBJ | rustc-objcopy |
| TARGET | match host |
| FLAGS | none |

# Functions

### `action`: Declaring a function
With `action`, you can declare a function. Simply write `action [name]`. **Remember to close it with `end.** Once declared, if you call it, it will run. It does not run it on declare, only on call.
# Statements

### `if`: Actions based on conditions
If you want to only do something if something else is a defined condition, you can use an `if` statement. For example, `if arg0 is "hello"`. These are closed using `end`. You can make multiple conditions using `if arg0 arg1 is "hello" or arg2 not "world"`. `if` supports `or`, `and`, `but not` (xor) for chaining conditions and `not` (!=), `is` (==) for the condition.

### `arg0 is arg0`: Conditioning nil values
When you need an `if` statement to fire if a value is nil, you type `thisvar is thisvar`.

# Actions

### `group`: Making groups
In CFBS, everything must be a group. Groups are structured simply, defined with `group [name]` and compile/assemble everything listed inside (example: `in kernel` and `with thing.rs`), and end with `end`. Indents are not required.

### `with`: Temporary flags and compilers
When you dont want to use the globally defined compiler or flags, you can use the `with` keyword.
To use a compiler, write `with CC [compiler]` or `with AS [assembler]`. To use a flag, write `with [flag]`.
The `with` keyword also works on groups, for example `group kernel with AS nasm CC clang`

### `link`: Linking groups
The `link` keyword (example: `link kernel`) will link every file in a group using a single `.ld` file. By default, it looks for `link/kernel.ld` when using the example of `link kernel` and groups them together using this. You can also use `link kernel as path/to/link.ld` to use a seperate linker file.

### `obj`: Using object copy
Object copy will take a linked group's `.elf` and turn it into a `.bin`. The output can be changed, for example `link kernel as img` to get a `.img` output instead of the standard `.bin`.

## `move`: Moving files
You can move files from the `out` folder (or what folder you defined for output) using the `move` keyword. For example: `move kernel to main` will copy the `kernel.img` from `project/out/done/kernel.img` to `project/kernel.img`

## `delete`: Removing files
You can remove a group of files, or a directory using this action. For example: `delete kernel` will remove all of the files for that group.

# Files

## `in`: Compiling/assembling directories
Using the `in` command will tell CFBS to compile/assemble everything inside of a directory, and can be customized by using `with` to use a different CC or AS, or add global flags for only that directory.

## `and`: Compiling/assembling single files
When you want to compile a single file not in the same directory as the rest of the group, you use `and`. `and` tells CFBS to add another file to the group in another directory, without compiling/assembling the entire directory, and can be customized the same way `in` can.

# Variables

### `is`: Defining global or standard variables
You can change a global variable, or define your own using the `is` keyword. For example `thisvar is 0` will return `0` if you ever recall `thisvar`. Or, if you write `CC is clang`, it will use clang for global compilation.

### `argX`: Command line arguments
You can access command line arguments using `arg`, for example, `arg0` will provide you the first command line variable (skipping CFBS program call and any CFBS-related arguments).

# IO calls

### `read`: Getting user input
If you want to get user input at any point during the script, you use `read`. An example is `input is read`.

### `print`: Sending out text
To output information to the user, you use `print`. An example is `print "hello" ThisVar`. If `ThisVar` is 12 for example, this will print `hello 12`. Strings and variables can be mixed together like this: `print ThisVar "and" ThatVar`.

# Shell commands
If you use a keyword that is not defined, it will attempt to execute that as a shell command. This allows you to use the shell inside of CFBS.

# Miscellaneous

### `end`: Closing statements
When you start a `group`, `if` or `action`, you need to close them using an `end`. Simply, at the end of that statement, write `end`.