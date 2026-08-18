# 📰 RataRSS

> A beautiful, minimal, themable, and customizable terminal RSS reader built with [Ratatui](https://ratatui.rs) in Rust, faithfully replicating the clean three-pane design of **NetNewsWire**.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)
![Ratatui](https://img.shields.io/badge/Ratatui-0.29-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

---

## ✨ Features

- 🖥️ **NetNewsWire 3-Pane Layout**:
  1. **Sidebar**: Smart Feeds (☀️ *Today*, 🔵 *All Unread*, ⭐ *Starred*, 📜 *All Articles*) + Collapsible Folders with unread count badges.
  2. **Article List**: Multi-line article cards displaying unread indicators (`●`), star markers (`★`), bold titles, summary previews, source tags, and relative timestamps (`1:20 PM`, `Yesterday`).
  3. **Reader View**: Clean typography, rich HTML/Markdown formatting (headings, blockquotes, code blocks, bullet points, links), and a dynamic reading progress indicator.
- 📐 **Interactive Resizable Panes**:
  - `<` / `>`: Adjust Sidebar width
  - `[` / `]`: Adjust Article List width
  - `+` / `-`: Adjust Reader View width
  - `=` / `Ctrl+R`: Reset to default layout proportions
  - `f` / `z`: Toggle Zen / Fullscreen mode on any pane
- 📥 **Standard OPML Import & Export**:
  - Full support for standard OPML 1.0 & 2.0 files with nested folders and metadata.
  - Interactive import/export modals in-app (`a` for add/import, `e` for export).
  - Headless CLI flags: `ratarss --import feeds.opml` and `ratarss --export backup.opml`.
- 🎨 **Themable & Highly Customizable**:
  - Built-in theme presets:
    - **NetNewsWire Dark** *(Default, matching macOS NetNewsWire dark aesthetic)*
    - **NetNewsWire Light**
    - **Catppuccin Mocha**
    - **Tokyo Night**
    - **Gruvbox Dark**
    - **Nord**
    - **Dracula**
    - **Minimal Monochrome**
  - Switch themes live with instant preview (`T` or `t`).
  - Persistent user configuration in `~/.config/ratarss/config.toml`.
- ⚡ **Fast Asynchronous Sync & Local SQLite Caching**:
  - Non-blocking background feed fetching with animated spinner (`⠋`).
  - Supports RSS 0.9x, RSS 2.0, Atom 1.0, and JSON Feed via `feed-rs`.
  - Automatic feed discovery from website URLs.
  - SQLite database persists all articles, read/unread states, stars, and folder tree.
- 🔍 **Realtime Search & Filter**:
  - Press `/` to search and filter article titles, snippets, authors, and sources in real time.
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
ratarss --theme "Tokyo Night"
```

---

## ⌨️ Keyboard Cheatsheet

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
| | `/` | Real-time search & filter articles |
| **Pane Resizing** | `<` / `>` | Decrease / Increase Sidebar width |
| | `[` / `]` | Decrease / Increase Article List width |
| | `+` / `-` | Decrease / Increase Reader width |
| | `=` | Reset layout to default percentages |
| | `f` or `z` | Toggle Fullscreen / Zen mode for active pane |
| **Management** | `a` | Add feed URL or Import OPML modal |
| | `e` | Export subscriptions to OPML modal |
| | `r` / `R` | Refresh current feed / Refresh all feeds |
| | `d` | Delete selected feed or folder |
| | `T` or `t` | Open interactive Theme Picker |
| | `?` or `F1` | Open Help modal |
| | `q` / `Ctrl+C` | Quit application |

---

## 📁 Project Architecture

```
src/
├── main.rs          # CLI argument parsing, terminal lifecycle, event loop
├── lib.rs           # Library exports
├── app.rs           # State machine, key/mouse event dispatching, background sync
├── model.rs         # Feed, Article, SidebarItem, SmartFeedKind, Filter models
├── storage.rs       # SQLite persistence, schema migrations, unread counters
├── fetcher.rs       # Async feed fetcher (RSS/Atom/JSON), auto-discovery, HTML cleaner
├── opml.rs          # OPML 1.0 & 2.0 XML parser and generator
├── reader.rs        # HTML/Markdown parser to styled Ratatui Text with word-wrap
├── theme.rs         # Theme palette engine with 8 preset styles (NetNewsWire, Tokyo Night, etc.)
├── config.rs        # Configuration loading and saving (~/.config/ratarss/config.toml)
├── sample_data.rs   # Curated starter feeds matching NetNewsWire layout
└── ui/
    ├── mod.rs         # 3-Pane split coordinator, status bar, and modal manager
    ├── sidebar.rs     # Left pane: Smart feeds, collapsible folder tree, unread badges
    ├── article_list.rs# Middle pane: NetNewsWire card format, snippets, timestamps, search
    ├── reader_view.rs # Right pane: Article header, typography, reader scroll, progress bar
    ├── modals.rs      # Popups: Add Feed, OPML Import/Export, Theme Picker, Help, Delete
    └── widgets.rs     # Custom widgets: Badges, progress bars, spinner animations
```

---

## 📄 License

Distributed under the MIT License.
