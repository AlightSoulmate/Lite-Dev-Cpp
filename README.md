# Lite Dev-C++

`Lite Dev-C++` is a lightweight C/C++ IDE for macOS and Windows, inspired by the workflow of Dev-C++.

Basic Functions: open a folder, edit C/C++ files, build the current source file, and run the produced executable without installing a heavy IDE.

## Download

Download the latest release for your operating system:

- macOS: download the `.dmg` or `.app.zip` package.
- Windows: download the `.zip` or installer package.

## Requirements

Lite Dev-C++ does not include a C/C++ compiler. Install one separately before building code.

macOS:

- Install Xcode Command Line Tools:

```sh
xcode-select --install
```

This provides `clang` and `clang++`. (clang does not support `bits/stdc++.h`, it needs to be created manually.)

Windows:

- Install a C/C++ compiler such as MSYS2 MinGW-w64 or LLVM.
- Set the compiler paths in Lite Dev-C++ if they are not available in `PATH`.

Example Windows compiler paths:

```text
C:\msys64\mingw64\bin\gcc.exe
C:\msys64\mingw64\bin\g++.exe
```

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

Open a project folder, edit the compiler fields in the top toolbar, then click `Save Config`.
The project settings are stored in:

```text
lite-dev-cpp.toml
```

Example:

```toml
[compiler]
c_compiler = "clang"
cpp_compiler = "clang++"
```

On Windows, these can be set to paths such as `C:\\msys64\\mingw64\\bin\\gcc.exe` and `C:\\msys64\\mingw64\\bin\\g++.exe`.

## Current Features

- Open a local folder as a project.
- Browse a simple file tree.
- Open, edit, and save `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`, `.hh`, and `.hxx` files.
- Configure C and C++ compiler commands.
- Build the current `.c`/`.cpp`/`.cc`/`.cxx` file into the project `build/` directory.
- Run the last built executable in a system terminal.
- Build and run the current source file with one toolbar action.
- Capture build stdout and stderr into the bottom output panel.
- Basic C/C++ editor highlighting, auto-paired brackets/quotes, Tab-to-spaces, and indentation after newlines.

## Build Behavior

- `.c` files use the configured C compiler, defaulting to `clang`.
- `.cpp`, `.cc`, and `.cxx` files use the configured C++ compiler, defaulting to `clang++`.
- Output executables are written to `build/`.
- Windows executables receive the `.exe` suffix.
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
The main supported release targets are macOS and Windows.
Linux may work through `eframe`, but it is not the primary release target yet.

## Release Packaging

End users should not need to install Rust or build from source. This repository includes a GitHub Actions workflow that builds release packages and attaches them to a GitHub Release.

The release workflow is defined here:

```text
.github/workflows/release.yml
```

It runs when a tag starting with `v` is pushed, for example `v0.1.0`.

What it builds:

- macOS universal app zip: `Lite-Dev-Cpp-macOS-universal.zip`
- Windows x86_64 zip: `Lite-Dev-Cpp-windows-x86_64.zip`

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
Lite-Dev-Cpp-windows-x86_64.zip
```

### Current Packaging Notes

- The macOS package is an unsigned `.app` inside a `.zip`.
- The Windows package is a `.zip` containing `Lite-Dev-Cpp.exe` and `README.md`.
- macOS users may need to approve the app manually in System Settings because it is not signed or notarized yet.
- Windows users may see SmartScreen warnings because the app is not code-signed yet.

Future release improvements:

- Add macOS signing and notarization.
- Add a `.dmg` package.
- Add a Windows installer.
- Add Windows code signing.

## Next Steps

- Move build and run commands to background tasks so the UI stays responsive.
- Add basic project restore on startup.
- Add editor quality-of-life features such as line numbers and find.
- Add diagnostics and semantic support through `clangd`.
- Replace the first-pass highlighter with `tree-sitter` for richer syntax handling.
- Add debugger integration in a later milestone.
- Add GitHub Actions builds for macOS and Windows release artifacts.
