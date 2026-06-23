<h1 align="center">LazyWSL 🐧</h1>

<p align="center">
  A lazy way to manage your WSL distros, right from your terminal.
</p>

<p align="center">
  <a href="https://github.com/0xDracula/LazyWSL/releases/latest"><img src="https://img.shields.io/github/v/release/0xDracula/LazyWSL" alt="Release"></a>
  <a href="https://github.com/0xDracula/LazyWSL/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows-0078D6" alt="Platform">
</p>

<p align="center">
  Built with Rust + Ratatui. Inspired by wsl-ui, LazyGit, and LazyDocker.
</p>

![overview](https://cdn.hackclub.com/019e3232-c684-7a20-a12d-f4c01f244e85/Screenshot%202026-05-16%20221031.png)

## Why LazyWSL?

Managing WSL through raw `wsl.exe` commands gets tedious fast. LazyWSL gives you a keyboard-first control panel for your distros: start, stop, snapshot, clone, and roll back, all without leaving the terminal.

- Keyboard-first workflow
- Fast distro management with live status
- Snapshot, rollback, and snapshot management
- Import / export for backup and migration
- Reusable custom actions
- A themed, responsive terminal UI

## Features

### Distro management
- Start / stop distros instantly
- Open a shell directly into a distro
- Set the default distro
- Clone a distro
- Multi-select (`Space`) and search (`/`)
- Live status, version, and size at a glance

### Snapshots
- Create a snapshot of any distro
- Roll back a distro to a snapshot
- **Snapshot Manager** (`S`): browse, delete, and prune snapshots, with per-distro and total disk usage

### Import / export
- Import a distro from a `.tar`
- Export a distro for backup or migration

### Productivity
- Custom reusable actions
- Themed notifications
- Built-in help menu (`?`)

## Installation

### Prebuilt binary (recommended)

Download the latest `lazywsl-windows-x86_64.exe` from the [Releases page](https://github.com/0xDracula/LazyWSL/releases/latest), then run it from a terminal. Each release includes a `.sha256` checksum to verify your download.

### From source

```bash
git clone https://github.com/0xDracula/LazyWSL
cd LazyWSL
cargo run --release
```

## Keybindings

**Navigation** — `↑`/`↓` move · `Space` multi-select · `/` search · `?` help

**Distro** — `Enter` shell · `r`/`t` run/stop · `d` default · `p` pin · `e`/`i` export/import · `a` actions · `n` clone

**Snapshots** — `z` snapshot · `b` rollback · `S` manager

**Danger** — `u` unregister · `s` shutdown · `q` quit

## Configuration

Configuration lives at:

```text
C:\Users\<YourUser>\AppData\Roaming\LazyWSL\
```

Example:

```json
{
  "timeouts": { "quickSecs": 5, "defaultSecs": 15, "longSecs": 60 },
  "refreshSecs": 2,
  "customActions": [
    { "name": "Update packages", "command": "sudo apt update && sudo apt upgrade -y" },
    { "name": "Show release", "command": "cat /etc/os-release" }
  ]
}
```

| Field | Description |
| --- | --- |
| `quickSecs` | Timeout for quick commands |
| `defaultSecs` | Default timeout for standard operations |
| `longSecs` | Timeout for long-running operations (import/export) |
| `refreshSecs` | How often distro state refreshes |
| `customActions` | Reusable commands available inside LazyWSL |

## Development

LazyWSL is a Windows tool, but most of its logic sits behind a service trait, so it can be developed on Linux/macOS using a built-in mock backend, no WSL required:

```bash
LAZYWSL_MOCK=1 cargo run
```

On non-Windows platforms the mock is used automatically.

## Roadmap

- [x] Import / export
- [x] Custom actions
- [x] Search
- [x] Multi-select
- [x] Distro cloning
- [x] Snapshot / rollback system
- [x] Snapshot manager
- [ ] Plugin system
- [ ] Configurable keymaps

## Contributing

PRs, issues, and feature requests are welcome. See the development section above to get started on any platform.

## License

[MIT](LICENSE)
