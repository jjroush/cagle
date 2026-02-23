# cagle

Promote Claude Code project permissions to your global settings.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![cagle](https://github.com/user-attachments/assets/23cf686f-57f6-4b3f-a53b-974ab6ed1364)

## Install

**Homebrew (macOS):**

```sh
brew install jjroush/tap/cagle
```

**From source:**

```sh
cargo install --git https://github.com/jjroush/cagle
```

## Usage

Run `cagle` in any directory that has a `.claude/settings.local.json`:

```sh
cagle
```

Select permissions from your project config and press Enter to add them to `~/.claude/settings.json`.

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Apply selected permission globally |
| `q` / `Esc` / `Ctrl-C` | Quit |

## License

[MIT](LICENSE)
