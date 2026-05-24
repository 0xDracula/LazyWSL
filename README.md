# <p align="center">LazyWSL 🐧</p>

![image](https://user-cdn.hackclub-assets.com/019e17c6-ece6-776f-bc2e-0777c7e81b9d/98b131cc-04bf-4da6-9378-9ce7b160c185.jpg)
A lazy way to manage your WSL distros, right from your terminal.
Built with Rust, Ratatui, inspired by wsl-ui, LazyGit, and LazyDocker.

## Overview 🐧

![image](https://cdn.hackclub.com/019e3232-c684-7a20-a12d-f4c01f244e85/Screenshot%202026-05-16%20221031.png)

## Why LazyWSL?

Managing WSL through raw `wsl.exe` commands becomes annoying fast.

LazyWSL provides:

- A keyboard-first workflow
- Fast distro management
- Import/Export workflows
- Reusable custom actions
- Terminal-native experience
- Smooth and Responsive ui interface

## Features ✨

### Distro Management

- Start/Stop distros instantly
- Open shell directly into selected distro
- Set default distro
- Multi Select [Space]
- Search distros

### Import / Export

- Import distros from `.tar`
- Export distros for backup or migration

### Productivity

- Custom reusable actions
- Keyboard-driven workflow
- Animated notifications
- Help menu with keybindings

## Installation

### From source

```bash
git clone https://github.com/0xDracula/LazyWSL

cd LazyWSL

cargo run --release
```

## Configuration

Configuration file is located at

```text
C:\Users\<YourUser>\AppData\Roaming\LazyWSL\
```

Example Configuration:

```json
{
  "timeouts": {
    "quickSecs": 5,
    "defaultSecs": 15,
    "longSecs": 60
  },
  "refreshSecs": 2,
  "customActions": [
    {
      "name": "Update packages",
      "command": "sudo apt update && sudo apt upgrade -y"
    },
    {
      "name": "Show release",
      "command": "cat /etc/os-release"
    }
  ]
}
```

### Config Fields

| Field | Description |
| ---- | ---- |
| `quickSecs` | Timeout for quick commands |
| `defaultSecs` | Default timeout for standard operations |
| `longSecs` | Timeout for long-running operations such as import/export |
| `refreshSecs` | How often distro state refreshes |
| `customActions` | Reusable commands avaliable inside LazyWSL |

## Roadmap

- [x] Import/Export
- [x] Custom Actions
- [x] Search
- [x] Multi-select
- [ ] Distro Cloning
- [ ] Snapshot / Rollback-system
- [ ] Plugin System
- [ ] Configurable Keymaps

## Contributing

PRs, issues, and feature requests are welcome.

---

## License

MIT

