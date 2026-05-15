# <p align="center">LazyWSL 🐧</p>
![image](https://user-cdn.hackclub-assets.com/019e17c6-ece6-776f-bc2e-0777c7e81b9d/98b131cc-04bf-4da6-9378-9ce7b160c185.jpg)
A lazy way to manage your WSL distros, right from your terminal.
Built with Rust, Ratatui, inspired by wsl-ui, LazyGit, and LazyDocker.

## Features ✨ (under development)


### Functions
- **List all WSL distros** with status and version (implemented)
- **Start/Stop distros** Directly from TUI (implemented)
- **Open shell** in any distro instantly (implemented)
- **Set default distro** (implemented)
- **Import/Export** distros as `.tar` files
- **Help menu** with all keybindings

## Overview 🐧

![image](https://cdn.hackclub.com/019e234c-1923-740e-817e-81f08a8d8494/Screenshot%202026-05-14%20004355.png)

## Config

Config file is located at `C:\Users\<YourUser>\AppData\Roaming\LazyWSL\` in this format:
```yaml
{
  "timeouts": {
    "quickSecs": 5,
    "defaultSecs": 15,
    "longSecs": 60
  },
  "refreshSecs": 2
}
```
quicksecs: timeout for quick commands

defaultSecs: timeout for default commands

longSecs: timeout for long time operations such as import/export

refreshSecs: how often the distro list gets refreshed