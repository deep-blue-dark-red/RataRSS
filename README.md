# 📰 RataRSS

> A fast, beautiful, themable, and customizable terminal RSS reader built with [Ratatui](https://ratatui.rs) in Rust.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)
![Ratatui](https://img.shields.io/badge/Ratatui-0.29-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

---

## ✨ Features

- 🖥️ **Modern 3-Pane TUI Layout**:
  1. **Sidebar**: Smart Feeds (☀️ *Today*, 🔵 *All Unread*, ⭐ *Starred*, 📜 *All Articles*) + Collapsible Folders with unread count badges.
  2. **Article List**: Multi-line article cards displaying unread indicators (`●`), star markers (`★`), bold titles, summary previews, source tags, and relative timestamps (`1:20 PM`, `Yesterday`).
  3. **Reader View**: Clean typography, rich HTML/Markdown formatting (headings, blockquotes, code blocks, bullet points, links), and a clean line position indicator (`Line 1/18`).
- ⚙️ **Interactive Configuration Menu (`/`)**:
  - Press `/` anywhere to toggle an interactive bottom popup menu.
  - Configure themes, auto-refresh on startup, refresh intervals, mark-read behavior, text wrapping, and pane layout proportions live with instant persistence.
  - Full support for **user-configurable keybindings** saved in `~/.config/ratarss/config.toml`.
- 🎨 **27 Built-in Themes & Full Palette Customization**:
  - Includes **RataRSS Dark** *(Default)*, **RataRSS Light**, **Catppuccin Mocha**, **Catppuccin Macchiato**, **Catppuccin Frappé**, **Catppuccin Latte**, **Tokyo Night**, **Tokyo Night Storm**, **Gruvbox Dark**, **Gruvbox Light**, **Nord**, **Dracula**, **Solarized Dark**, **Solarized Light**, **Rosé Pine**, **Rosé Pine Dawn**, **Rosé Pine Moon**, **Monokai Pro**, **One Dark**, **GitHub Dark**, **GitHub Light**, **Kanagawa**, **Everforest Dark**, **Everforest Light**, **Cyberpunk Neon**, **Horizon**, and **Minimal Monochrome**.
  - Interactive theme picker (`T` or `t`) with live search, smooth scrolling, and dynamic window sizing.
- ⚡ **Optimized Performance & Battery Friendly**:
  - Reactive event loop that sleeps when idle, eliminating wasteful CPU cycles while maintaining snappy 100ms response during live animations and syncing.
- 📥 **Standard OPML Import & Export**:
  - Full support for standard OPML 1.0 & 2.0 files with nested folders and metadata.
  - Interactive import/export modals in-app (`a` for add/import, `e` for export).
  - Headless CLI flags: `ratarss --import feeds.opml` and `ratarss --export backup.opml`.
- 🔄 **Robust Multi-Feed Background Sync & Local SQLite Caching**:
  - Concurrent non-blocking background feed fetching with connection timeouts and auto-completion.
  - Automatic feed discovery from website URLs.
  - SQLite database persists all articles, read/unread states, stars, and folder tree.
- 🔍 **Realtime Search & Filter**:
  - Press `Ctrl+F` (or customize in keybindings) to filter article titles, snippets, authors, and sources in real time.
- 🌐 **Browser & Clipboard Integration**:
  - Press `o` or `Enter` in the reader to open the original article in your default browser.
  - Press `y` to copy article link to your clipboard.

---

## 🚀 Quick Start

### Build and Run
```bash
# Clone the repository
git clone https://github.com/ratarss/ratarss.git
cd ratarss

# Run with default curated feeds
cargo run

# Or build the release binary
cargo build --release
./target/release/ratarss
```

### CLI Options
```bash
# Import subscriptions from an OPML file
ratarss --import ~/Downloads/subscriptions.opml

# Export subscriptions to OPML
ratarss --export my_feeds_backup.opml

# Add a feed directly from the command line
ratarss --add "https://news.ycombinator.com/rss" --folder "Tech"

# Launch with a specific theme
ratarss --theme "Catppuccin Mocha"
```

---

## ⌨️ Default Keyboard Reference

*(All keybindings can be customized in `~/.config/ratarss/config.toml` or viewed via the `/` config menu)*

| Category | Keybinding | Action |
| :--- | :--- | :--- |
| **Navigation** | `Tab` / `Shift+Tab` | Cycle focus between Sidebar, Article List, and Reader |
| | `1` / `2` / `3` | Direct jump to Sidebar (`1`), Article List (`2`), Reader (`3`) |
| | `h` / `l` or `←` / `→` | Move focus left / right |
| | `j` / `k` or `↓` / `↑` | Navigate items in list or scroll article |
| | `Space` | Page down reader / advance to next unread |
| | `g` / `G` | Jump to top / bottom of active view |
| **Article Actions** | `m` | Toggle Read / Unread status |
| | `Shift+M` | Mark all articles in current view as read |
| | `s` | Toggle Star / Bookmark |
| | `o` / `Enter` | Open article in default web browser |
| | `y` | Copy article URL to clipboard |
| | `Ctrl+F` / `Ctrl+S` | Real-time search & filter articles |
| **Configuration** | `/` | Toggle interactive Configuration Menu popup |
| | `T` or `t` | Open interactive Theme Picker (27 themes) |
| **Pane Resizing** | `<` / `>` | Decrease / Increase Sidebar width |
| | `[` / `]` | Decrease / Increase Article List width |
| | `+` / `-` | Decrease / Increase Reader width |
| | `=` | Reset layout to default percentages |
| | `f` or `z` | Toggle Fullscreen / Zen mode for active pane |
| **Feed Management** | `a` | Add feed URL or Import OPML modal |
| | `e` | Export subscriptions to OPML modal |
| | `r` / `R` | Refresh current feed / Refresh all feeds |
| | `d` | Delete selected feed or folder |
| | `?` or `F1` | Open Help modal |
| | `q` / `Ctrl+C` | Quit application |

---

## 📄 License

Distributed under the MIT License.
