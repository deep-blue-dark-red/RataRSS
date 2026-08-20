# Ideas heap

Unsorted-but-grouped list of things RataRSS does not do yet. Each entry says
what is missing and, where it matters, what the code already has that would
make it cheap. Nothing here is a commitment.

Status legend: **done** — shipped; everything else is open.

---

## Navigation & reading flow

- **Next / previous article from any pane** — **done.** `J`/`K` (bindable as
  `next_article` / `prev_article`) move through the article list from wherever
  focus is, and in zen mode `j`/`k` do the same, because the list is off-screen
  there and the reader was otherwise a dead end. Split view keeps `j`/`k` as a
  line scroll.
- **Reader scroll stops at the end of the article** — **done.** The offset used
  to run past the last line (`G` added a flat 100 rows), leaving a blank pane
  that took as many presses to escape. `App::max_reader_scroll` clamps against
  the formatted line count and the recorded text area.
- **Next unread** — the classic RSS verb: skip read items in the current view,
  optionally rolling into the next feed that has unread articles. Nothing does
  this today.
- **`space` should advance, not just scroll** — `space_advance` pages the reader
  and stops at the bottom. The name promises "page down, then jump to the next
  article", which is what most readers do.
- **Half-page scroll** (`ctrl+d` / `ctrl+u`) in the reader and the article list.
- **`g` / `G` scoped to the article** in the reader (top / bottom of the
  document) rather than sharing the pane-level jump semantics.
- **Keyboard link following** — considered and declined for now.
  `FormattedArticle::hits` already records every link region, so numbered hints
  (`f` then a digit) or `Tab`-cycling would be cheap if it ever comes back up.

## Settings that exist but do nothing

Each of these is either a feature to build or a line to delete.

- **`refresh_interval_minutes`** — **done.** The interval is now honoured: a
  scheduled `refresh_all_feeds()` runs from `on_tick` when the interval has
  elapsed, skipped while a sync is already in flight, and any manual `r`/`R`
  restarts the clock.
- **`wrap_article_text`** — toggled in the settings menu, never consulted;
  `render_article_to_text` always wraps. The feature worth building behind it is
  a **maximum reading column** (say 80 cells, centered), which zen mode needs
  badly on a wide terminal.
- **`max_articles_per_feed`** — set to 200 and never read. Behind it: a
  **retention policy** — prune old, read, unstarred articles and `VACUUM` — or
  the database grows without bound.
- **`Feed::custom_icon`** — in the model and the schema, never rendered and not
  editable anywhere.

## Feed & subscription management

- **There is no edit path at all.** Feeds can be added and deleted, nothing
  else. Missing: rename feed, move feed between folders, create / rename folder,
  change a feed's URL.
- **Feed error details** — `Feed::error` drives a badge in the sidebar, but the
  message itself is unreachable. A detail view (last sync time, last error,
  article count) for the selected feed.
- **Pause / mute a feed** without unsubscribing.
- **Undo** for unsubscribe and for mark-all-read. Both are one keypress and
  irreversible.

## Sync

- **Conditional GET (ETag / `If-Modified-Since`)** — **done.** The validators
  from each successful fetch are stored on the feed and replayed on the next
  request; a 304 comes back as `FetchOutcome::NotModified` and writes nothing,
  parses nothing and triggers no reload. Feeds that send no validators simply
  keep getting 200s. Not covered yet: recording a "last checked" time on the
  304 path (deliberately left out to keep it free of writes), and the
  discovery fallback drops validators because the URL it read is not the URL
  the next fetch will ask for.
- **Per-feed backoff** after repeated failures, instead of retrying a dead feed
  every cycle at full rate.
- **"Last synced" indicator** in the status bar. The 15-second failsafe
  currently reports "Feed sync completed." even when the round timed out.
- **Server sync** (Miniflux / FreshRSS / Fever / Google Reader API) — the big
  one. Turns this from a standalone reader into a client whose read state
  follows the user between machines.

## Search & filtering

- **Global search backed by SQLite FTS5** — search is currently scoped to the
  articles loaded for the current view and cannot see article bodies, which are
  never held in memory. FTS would search everything and not depend on which feed
  is selected.
- **Filter operators** — `is:unread`, `is:starred`, `feed:foo`; or at minimum an
  "unread only" toggle for the article list and a "hide read feeds" toggle for
  the sidebar.
- **Sort options** — oldest first, group by day, group by feed.

## Content

- **Full-text fetch** — many feeds ship truncated summaries. Fetching the
  article URL and extracting readable content is the feature that makes a TUI
  reader worth living in.
- **Images** — stripped silently today. At minimum render `[image: alt text]`;
  optionally inline via the kitty or sixel protocols.
- **Reading time / word count** in the reader header.
- **Copy as Markdown** (title + URL) and export article to a file. `y` copies
  the bare URL.

## Bulk & organization

- **Tags / labels**, and a **read-later queue** distinct from stars.
- **Multi-select** in the article list for bulk star / read / delete.
- **Collapse all / expand all** folders in the sidebar.

## CLI & automation

- **`ratarss refresh`** for cron, and **`ratarss list --unread --json`** for
  scripting. The headless surface today is only `--import`, `--export`, `--add`.
- **`--db-path` / environment override** — the database location is fixed by
  `ProjectDirs` with no escape hatch, which is also why the integration tests
  open the real user database instead of a fixture.

## Interaction polish

- **Scrollbars** for the sidebar and article list; the reader has a position
  indicator, the other panes have nothing.
- **Drag to resize panes** with the mouse, middle-click to open in browser.
- **Quit confirmation** while a sync is in flight.
</content>
</invoke>
