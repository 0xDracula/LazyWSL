# Contributing to LazyWSL 🐧

First off, thanks for considering contributing to LazyWSL! LazyWSL is a keyboard-first, terminal-native manager for WSL distributions. All type of PRs are welcomed including UI improvements, new features, etc.

---

## getting started!

### prerequisites

to build and run LazyWSL locally, you will need:

- **Rust** => the latest stable version (install via [rustup.rs](https://rustup.rs))
- **Windows with WSL installed** => since this tool uses `wsl.exe`
- **Git** => to clone and manage your changes

### dev setup

#### linux/macos

LazyWSL is a Windows tool, but most of the logic (state, parsing, UI, etc) is platform independent and sits behind the `WSLService` trait. you can develop the whole thing on linux/macos using the built-in mock backend

run the tui with

```LAZYWSL_MOCK=1 cargo run
```

on non-windows platforms that mock is automatically used, so plain `cargo run` works too! on windows set `LAZYWSL_MOCK=1` to force mock (useful for UI testing)

1. fork
2. clone your fork
3. build and run (cargo run)


## development standards

to maintain code quality and consistency, please follow these

### 1. code style

we use the standard rust formatting, before submitting a PR run cargo fmt

### 2. linting

run clippy to catch common mistakes and idiomatic improvements: `cargo clippy -- -D warnings`

## PR process

1. **create a branch** => use a descriptive name
2. **commit messages** => follow a clear format (e.g., `feat: add supprt for x`)
3. **update documentation when needed**
4. **self review** => ensure you code is clean, commented where necessary.
5. **open the PR**

## reporting issues

If you find a bug or have a feautre request [open an issue](https://github.com/0xDracula/LazyWSL/issues)
