# Lite Dev-C++

`Lite Dev-C++` is a lightweight C/C++ editor for macOS, written in Rust, inspired by Dev-C++.

Basic Functions: open a folder, edit C/C++ files, build the current source file, and run the produced executable without installing a heavy IDE.

## Download

Download the latest release for your operating system:

- macOS: download the `.dmg` or `.app.zip` package.

## Requirements

Lite Dev-C++ does not include a C/C++ compiler. Install one separately before building code.

macOS:

- Install Xcode Command Line Tools:

```sh
xcode-select --install
```

This provides `clang` and `clang++`. (clang does not support `bits/stdc++.h`, it needs to be created manually.)

## Basic Usage

1. Launch Lite Dev-C++.
2. Click `Open Folder` and choose a folder containing C/C++ files.
3. Select a `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, or `.hpp` file from the file tree.
4. Edit the file in the center editor.
5. Click `Save File`.
6. Click `Build` to compile the current source file.
7. Click `Run` to run the last built executable in a system terminal.
8. Click `Build & Run` to compile and run in one step.

## Compiler Configuration

Edit the compiler fields in the top toolbar, then click `Save Config`.
The compiler settings are stored in the user's app config directory, not in the opened project folder.

On macOS, the config file is:

```text
~/Library/Application Support/dev.LiteDevCpp.Lite-Dev-C++/config.toml
```

Example:

```toml
[compiler]
c_compiler = "clang"
cpp_compiler = "clang++"
```

## Current Features

- Open a local folder as a project.
- Browse a simple file tree.
- Use file tree context menus for opening, revealing, copying paths, creating, renaming, deleting, and refreshing items.
- Open, edit, and save `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`, `.hh`, and `.hxx` files.
- Configure C and C++ compiler commands.
- Build the current `.c`/`.cpp`/`.cc`/`.cxx` file into an executable named `a` beside the source file.
- Run the last built executable in a system terminal.
- Build and run the current source file with one toolbar action.
- Capture build stdout and stderr into the bottom output panel.
- Basic C/C++ editor highlighting, auto-paired brackets/quotes, Tab-to-spaces, and indentation after newlines.

## Build Behavior

- `.c` files use the configured C compiler, defaulting to `clang`.
- `.cpp`, `.cc`, and `.cxx` files use the configured C++ compiler, defaulting to `clang++`.
- Output executables are written as `a` in the current source file's folder.
- Building another source file in the same folder overwrites that same `a` executable.
- `Run` and `Build & Run` launch a system terminal so programs using `cin`, `scanf`, or other stdin reads can wait for user input.

## Status

Lite Dev-C++ is currently a minimal working version, not a full IDE.
It does not yet include a debugger, project templates, full code completion, or integrated `clangd`.