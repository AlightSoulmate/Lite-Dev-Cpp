# Lite Dev-C++

`Lite Dev-C++` is a lightweight C/C++ IDE for macOS, inspired by the workflow of Dev-C++.

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

## For Developers

Install Rust, then run from the repository root:

```sh
cargo run
```

Check and build locally:

```sh
cargo fmt
cargo check
cargo build
```

The app is written in Rust with `egui`/`eframe`.
The supported release target is macOS.

## Release Packaging

End users should not need to install Rust or build from source. This repository includes a GitHub Actions workflow that builds release packages and attaches them to a GitHub Release.

The release workflow is defined here:

```text
.github/workflows/release.yml
```

It runs when a tag starting with `v` is pushed, for example `v0.1.0`.

What it builds:

- macOS universal app zip: `Lite-Dev-Cpp-macOS-universal.zip`

### Publish a Release

After the repository has been pushed to GitHub, create a release like this:

```sh
git status
git add .
git commit -m "Prepare Lite Dev-C++ release"
git push origin main
git tag v0.1.0
git push origin v0.1.0
```

Then open the repository on GitHub and go to:

```text
Actions -> Release
```

When the workflow finishes, GitHub will create a Release page with downloadable files:

```text
Lite-Dev-Cpp-macOS-universal.zip
```

### Current Packaging Notes

- The macOS package is an unsigned `.app` inside a `.zip`.
- macOS users may need to approve the app manually in System Settings because it is not signed or notarized yet.

Future release improvements:

- Add macOS signing and notarization.
- Add a `.dmg` package.

## Next Steps

- Move build and run commands to background tasks so the UI stays responsive.
- Add basic project restore on startup.
- Add editor quality-of-life features such as line numbers and find.
- Add diagnostics and semantic support through `clangd`.
- Replace the first-pass highlighter with `tree-sitter` for richer syntax handling.
- Add debugger integration in a later milestone.
- Add signed and notarized macOS release artifacts.
