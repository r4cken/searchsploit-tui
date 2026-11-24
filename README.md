# searchsploit-tui

searchsploit-tui is a terminal user interface for Searchsploit written in Rust using Ratatui.

The tool queries a local copy of the exploitdb database and presents a list of exploits matching your query. A preview pane shows the exploit source code with syntax highlighting and
you can copy any selected exploit to your current working directory.

<img width="1913" height="1035" alt="2026-11-19-101223_hyprshot" src="https://github.com/user-attachments/assets/e922a585-9944-43f5-be16-56c4d201e235" />

## Prerequisites
Requires Linux, Rust 1.85+ (edition 2024), and a local copy of the exploit database. Searchsploit is not distributed with this program.

Clone the exploit database repository and put Searchsploit on your PATH
```bash
sudo git clone https://gitlab.com/exploit-database/exploitdb.git /opt/exploitdb
sudo ln -sf /opt/exploitdb/searchsploit /usr/local/bin/searchsploit
```
NOTE: If you're running Kali Linux, Parrot OS, BlackBox or BlackArch then you already have a copy of exploitdb on your system, so you can skip this step.

## Installing searchsploit-tui
```bash
cargo install --git https://github.com/r4cken/searchsploit-tui
```

Assumes `~/.cargo/bin` is on your PATH (rustup sets this up).

## Building from source
```bash
git clone https://github.com/r4cken/searchsploit-tui.git
cd searchsploit-tui
cargo build --release
```

The built binary can be found at `target/release/searchsploit-tui`. Place it somewhere on your PATH.

## Interacting with the terminal user interface

| Key | Action |
|---|---|
| `ESC` | exit search mode
| `/` or `i` | enter search mode where you specify the search query. Press enter to search |
| `j` / `k`, up/down arrow keys | next/previous entry, or scroll up/down in the preview pane when it has focus |
| `Tab`, `h`/`l`, left/right arrow keys | switch focus between the search results and preview pane |
| `y` | copy the path to the selected exploit on disk to the clipboard |
| `m` | mirror a selected exploit to the current directory |
| `q` or `Ctrl+C` | quit |

## Known limitations
Copying a path to the clipboard for the selected exploit on disk fails when running searchsploit-tui in a headless Linux distribution because there's no display server running.
