# <p align="center">LazyWSL 🐧</p>
![image](https://user-cdn.hackclub-assets.com/019e17c6-ece6-776f-bc2e-0777c7e81b9d/98b131cc-04bf-4da6-9378-9ce7b160c185.jpg)
A lazy way to manage your WSL distros, right from your terminal.
Built with Rust, Ratatui, inspired by wsl-ui, LazyGit, and LazyDocker.

## Features ✨


### Functions
- **List all WSL distros** with status and version
- **Start/Stop distros** Directly from TUI
- **Open shell** in any distro instantly
- **Set default distro**  instantly
- **Import/Export** distros as `.tar` files
- **Custom Actions** run reusable commands against any selected distro
- **Help menu** with all keybindings

## Overview 🐧

![image](https://cdn.hackclub.com/019e3232-c684-7a20-a12d-f4c01f244e85/Screenshot%202026-05-16%20221031.png)

## Config

Config file is located at `C:\Users\<YourUser>\AppData\Roaming\LazyWSL\` in this format:
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
`quicksecs`: timeout for quick commands

`defaultSecs`: timeout for default commands

`longSecs`: timeout for long time operations such as import/export

`refreshSecs`: how often the distro list gets refreshed

`customActions`: reusable shell commands