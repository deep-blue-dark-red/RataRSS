# 📰 RataRSS

> A fast, beautiful, themable, and customizable terminal RSS reader built with [Ratatui](https://ratatui.rs) in Rust.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)
![Ratatui](https://img.shields.io/badge/Ratatui-0.29-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

---

## ✨ Features

- 🖥️ **Minimal 3-Pane TUI Layout**:
  0. **No top bar**: the panes start at the first row. All chrome lives in a single status line — brand bottom-left, sync/toast state beside it, layout and theme on the right.
  1. **Sidebar**: Smart Feeds (☀️ *Today*, 🔵 *All Unread*, ⭐ *Starred*, 📜 *All Articles*) + Collapsible Folders with unread count badges.
  2. **Article List**: Multi-line article cards displaying unread indicators (`●`), star markers (`★`), bold titles, summary previews, source tags, and relative timestamps (`1:20 PM`, `Yesterday`).
  3. **Reader View**: Clean typography, rich HTML/Markdown formatting (headings, blockquotes, code blocks, bullet points, links), and a clean line position indicator (`1/18`).
  - **Declutter to taste**: adjustable content padding, article spacing in half-row steps, every icon and emoji removable, and shortcut hints hidden by default — toggle them with `??` or from `/`.
  - **Clickable links**: anchors in article HTML are drawn underlined and open in your browser on click, as does the article's own URL.
- ⚙️ **Interactive Configuration Menu (`/`)**:
  - Press `/` anywhere to toggle an interactive bottom popup menu.
  - Configure themes, auto-refresh on startup, refresh intervals, mark-read behavior, text wrapping, icons on/off, content padding, status-bar shortcut hints, and pane layout proportions live with instant persistence.
  - Full support for **user-configurable keybindings** saved in `~/.config/ratarss/config.toml`.
- 🎨 **27 Built-in Themes & Full Palette Customization**:
  - Includes **RataRSS Dark** *(Default)*, **RataRSS Light**, **Catppuccin Mocha**, **Catppuccin Macchiato**, **Catppuccin Frappé**, **Catppuccin Latte**, **Tokyo Night**, **Tokyo Night Storm**, **Gruvbox Dark**, **Gruvbox Light**, **Nord**, **Dracula**, **Solarized Dark**, **Solarized Light**, **Rosé Pine**, **Rosé Pine Dawn**, **Rosé Pine Moon**, **Monokai Pro**, **One Dark**, **GitHub Dark**, **GitHub Light**, **Kanagawa**, **Everforest Dark**, **Everforest Light**, **Cyberpunk Neon**, **Horizon**, and **Minimal Monochrome**.
  - Interactive theme picker (`T` or `t`) with live search, smooth scrolling, and dynamic window sizing.
- ⚡ **Optimized Performance & Battery Friendly**:
  - Reactive event loop that sleeps when idle and coalesces input bursts, so held keys and scroll wheels never queue up behind redraws.
  - Nothing is copied to draw a frame: panes borrow the article list, and the reader keeps its formatted text cached until the article, width or theme changes.
  - Article bodies stay in SQLite. The list reads **no compressed column at all** — it draws a stored plain-text `snippet` — and only the single article on screen is ever decompressed.
  - Views sort on a stored `sort_ts` key with covering indexes, so SQLite walks an index instead of sorting the whole table. On a real 13k-article database, opening *All Unread* went from ~177 ms (and 24 MB decompressed) to ~9 ms (and none).
  - Read/star changes patch state in place instead of re-querying the database.
- 📥 **Standard OPML Import & Export**:
  - Full support for standard OPML 1.0 & 2.0 files with nested folders and metadata.
  - Interactive import/export modals in-app (`a` for add/import, `e` for export).
  - Headless CLI flags: `ratarss --import feeds.opml` and `ratarss --export backup.opml`.
- 🔄 **Unlimited Persistent Storage with Zstandard (zstd) Compression**:
  - Embedded **SQLite engine** with Write-Ahead Logging (`WAL` mode) and indexes for instant sub-millisecond queries.
  - Transparent **Zstandard (zstd level 3)** compression on all article bodies and summaries (75–85% disk space reduction).
  - Millions of articles can be stored and archived persistently with negligible disk usage and gigabytes/sec decompression.
  - A single pooled connection with cached prepared statements; the legacy compression migration runs once and then marks itself done.
  - Automatic migration compresses any existing legacy articles seamlessly on startup.
- 🌐 **Robust Multi-Feed Background Sync**:
  - Concurrent non-blocking background feed fetching with connection timeouts and auto-completion.
  - Automatic feed discovery from website URLs.
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
