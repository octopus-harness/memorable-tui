# memorable-tui

<p align="center">
  <strong>A fast, keyboard-driven terminal user interface for the <a href="https://github.com/octopus-harness/memorable">Memorable</a> persistent AI context and memory studio.</strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-2021_Edition-dea584?style=flat-square&logo=rust&logoColor=white" alt="Rust 2021"></a>
  <a href="https://ratatui.rs"><img src="https://img.shields.io/badge/Ratatui-0.28-111111?style=flat-square" alt="Ratatui"></a>
  <a href="https://github.com/crossterm-rs/crossterm"><img src="https://img.shields.io/badge/Crossterm-0.28-blue?style=flat-square" alt="Crossterm"></a>
  <img src="https://img.shields.io/badge/Architecture-Local--First-22c55e?style=flat-square" alt="Local-First Architecture">
  <a href="https://github.com/octopus-harness/memorable"><img src="https://img.shields.io/badge/Memorable-Rails_8-D30001?style=flat-square&logo=rubyonrails&logoColor=white" alt="Memorable Engine"></a>
  <a href="https://github.com/pgvector/pgvector"><img src="https://img.shields.io/badge/pgvector-1024--dim_HNSW-8B5CF6?style=flat-square&logo=postgresql&logoColor=white" alt="pgvector"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License: MIT"></a>
</p>

---

A fast, keyboard-driven terminal user interface (TUI) for [Memorable](https://github.com/octopus-harness/memorable): the local-first persistent AI memory and developer workbench built with [Ratatui](https://ratatui.rs) and [Crossterm](https://github.com/crossterm-rs/crossterm).

## Features

- **Flat Workspace Architecture (Violet & Emerald Theme)**:
  - **No Project Friction**: Direct, unnested access to all context memories, instructions, and rulesets.
  - **Pane 1 `[1] Scopes & Pinned`**: Virtual scopes (All Memories, ★ Pinned, Vector Embedded, Unembedded, Recent) and direct-access pinned memories.
  - **Pane 2 `[2] Memories`**: Virtualized memories list with pin markers (`★`), vector embedding indicators (`[EMB]`), context attachment counters (`📎`), checklist progress badges (`[done/total]`), and timestamps.
  - **Pane 3 `[3] Memory Detail [Preview | Raw]`**: Dual-mode Markdown renderer, interactive checklist toggler, and pinned nearest semantic neighbors panel.
- **Vector Search & Semantic Embeddings (pgvector HNSW)**:
  - `V`: Trigger immediate 1024-dimensional vector embedding generation for the active memory.
  - **Nearest Neighbors**: Automatically queries and displays top semantically related memories with cosine match percentages directly at the bottom of the Detail pane.
  - **Auto Search (`/`)**: Real-time search defaulting to server-side vector semantic match with automatic keyword fallback.
- **Dual Markdown Modes & Smart Checklists**:
  - `v` / `m`: Toggle between **Styled Preview Mode** (headings, code blocks, blockquotes, tables, bullet lists) and **Raw Markdown Mode** (with line numbers).
  - If a memory contains `- [ ]` / `- [x]` checklist tasks, `j` / `k` navigates between items and `Space` / `Enter` toggles completion state back to the server.
- **Host Resolution & Host Switcher**:
  - **Environment default**: Resolves `MEMORABLE_REMOTE_URL` (or `MEMORABLE_URL`) when set in the shell (e.g. `export MEMORABLE_REMOTE_URL="http://memorable.internal:3000"`).
  - **Fallback**: If `MEMORABLE_REMOTE_URL` is not set, defaults to `http://localhost:3000`.
  - **CLI overrides**: `--url <URL>`, `--host <URL>`, `--remote`, and `--local` (`http://localhost:3000`) always take precedence over the environment.
  - **Host Switcher**: `H` or `u` allows switching between presets or connecting to custom URLs on the fly (locked when `MEMORABLE_REMOTE_URL` is set, unless `--allow-switch` is passed).
- **CRUD, Quick Append & External `$EDITOR`**:
  - `a`: Quick Append modal to append markdown content or tasks (`- [ ]`) under an optional section header.
  - `n` / `e`: Create or edit memory title, initial body, and pin state.
  - `d`: Delete memory or attachment with confirmation.
  - `p`: Toggle pinned state.
  - `K` / `J`: Reorder pinned memories up or down when in the Pinned scope.
  - `E`: Launch external `$EDITOR` (e.g. `nvim`, `vim`, `nano`) on memory body, synchronizing changes on exit.
  - `A`: Context Attachment Manager to inspect, upload, download, or delete attached files.
  - `c` / `Y`: Copy full markdown body to system clipboard.
  - `y`: Yank active checklist item text or memory title to clipboard.

## Installation & Running

```bash
# Build binary
cargo build --release

# Run against default host (http://localhost:3000 if MEMORABLE_REMOTE_URL is not set)
cargo run

# Specify explicit host URL
cargo run -- --url http://localhost:3000

# Target local instance explicitly even if MEMORABLE_REMOTE_URL is set
cargo run -- --local

# Allow in-app Host Switcher when MEMORABLE_REMOTE_URL is set
cargo run -- --allow-switch
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1`, `2`, `3` | Direct focus to Pane 1 (Scopes), 2 (Memories), 3 (Detail) |
| `Tab` / `BackTab` | Cycle focus forward / backward between panes |
| `h` / `l` | Move focus left / right |
| `j` / `k` | Move selection down / up in focused pane |
| `Enter` | Jump from sidebar to memories list, or memories list to detail |
| `v` / `m` | Toggle between Styled Preview and Raw Markdown modes |
| `Space` / `Enter` | Toggle active checklist item `[ ]` ⇄ `[x]` (in Pane 3) |
| `a` | Quick-append markdown text or task to memory |
| `V` | Trigger 1024-dim vector embedding generation |
| `E` | Open memory body in external `$EDITOR` |
| `J` / `K` | Scroll detail document down / up by 5 lines (or reorder pinned items in Pane 2) |
| `g` / `G` | Jump to top / bottom of document or list |
| `c` / `Y` | Copy entire Markdown body to system clipboard |
| `y` | Yank active task text or memory title to system clipboard |
| `H` / `u` | Open Host Switcher dialog |
| `/` | Real-time vector and keyword search (`Esc` to clear) |
| `n` | Create new memory |
| `e` | Edit memory metadata |
| `d` | Delete memory or attachment with confirmation |
| `p` | Toggle pinned status on selected memory |
| `A` | Open context attachments manager |
| `r` | Refresh workspace and nearest neighbors from server |
| `?` | Show help modal |
| `q` | Quit application |
