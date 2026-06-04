
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
