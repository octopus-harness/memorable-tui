#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::KeyCode;

use crate::api::ApiClient;
use crate::checklist::{
    append_to_memory, parse_checklist_items, toggle_checklist_item_by_index, ChecklistItem,
};
use crate::config::{get_preset_hosts, Config};
use crate::models::{
    Memory, MemoryDetail, MemoryInput, NeighborResult, Scope, Stats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Sidebar,
    Memories,
    Detail,
}

impl Pane {
    pub fn next(self) -> Self {
        match self {
            Pane::Sidebar => Pane::Memories,
            Pane::Memories => Pane::Detail,
            Pane::Detail => Pane::Sidebar,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Pane::Sidebar => Pane::Detail,
            Pane::Memories => Pane::Sidebar,
            Pane::Detail => Pane::Memories,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Preview,
    Raw,
}

impl ViewMode {
    pub fn toggle(self) -> Self {
        match self {
            ViewMode::Preview => ViewMode::Raw,
            ViewMode::Raw => ViewMode::Preview,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Preview => "Preview",
            ViewMode::Raw => "Raw",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeItem {
    Scope(Scope),
    PinnedMemory(Memory),
}

#[derive(Debug, Clone)]
pub enum DeleteTarget {
    Memory(Memory),
    Attachment {
        memory_id: i64,
        attachment_id: i64,
        filename: String,
    },
}

#[derive(Debug, Clone)]
pub enum Modal {
    HostSwitcher {
        cursor: usize,
        custom_input: String,
        entering_custom: bool,
    },
    Search {
        input: String,
    },
    QuickAppendMemory {
        content_input: String,
        section_input: String,
        as_task: bool,
        focus_field: usize, // 0: content, 1: section, 2: as_task
    },
    MemoryForm {
        id: Option<i64>,
        title_input: String,
        body_input: String,
        pinned: bool,
        focus_field: usize, // 0: title, 1: body, 2: pinned
    },
    ConfirmDelete {
        target: DeleteTarget,
    },
    Attachments {
        cursor: usize,
        upload_input: Option<String>,
        dest_input: Option<String>,
    },
    Help,
}

pub struct App {
    pub config: Config,
    pub client: ApiClient,
    pub should_quit: bool,
    pub pending_editor: Option<(i64, PathBuf)>, // memory_id, temp_path
    pub status: Option<(String, bool)>,         // (message, is_error)

    // Layout & Navigation
    pub focused_pane: Pane,
    pub view_mode: ViewMode,
    pub modal: Option<Modal>,

    // Data
    pub stats: Stats,
    pub preset_hosts: Vec<String>,
    pub pinned_memories: Vec<Memory>,
    pub memories: Vec<Memory>,
    pub current_memory: Option<MemoryDetail>,
    pub current_neighbors: Vec<NeighborResult>,
    pub checklist_items: Vec<ChecklistItem>,

    // Selections
    pub sidebar_selected: usize,
    pub memories_selected: usize,
    pub detail_selected_checklist: Option<usize>,
    pub detail_scroll: usize,

    // Search query
    pub search_query: Option<String>,

    pub clipboard: Option<Clipboard>,
}

pub fn sort_memories(memories: &mut [Memory]) {
    memories.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.id.cmp(&b.id))
    });
}

impl App {
    pub fn new(config: Config) -> Self {
        let client = ApiClient::new(&config);
        let clipboard = Clipboard::new().ok();

        let mut app = Self {
            config,
            client,
            should_quit: false,
            pending_editor: None,
            status: None,

            focused_pane: Pane::Memories,
            view_mode: ViewMode::Preview,
            modal: None,

            stats: Stats::default(),
            preset_hosts: get_preset_hosts(),
            pinned_memories: Vec::new(),
            memories: Vec::new(),
            current_memory: None,
            current_neighbors: Vec::new(),
            checklist_items: Vec::new(),

            sidebar_selected: 0,
            memories_selected: 0,
            detail_selected_checklist: None,
            detail_scroll: 0,

            search_query: None,
            clipboard,
        };

        app.reload_all();
        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = Some((msg.into(), is_error));
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn switch_host(&mut self, new_url: &str) {
        let normalized = crate::config::normalize_url(new_url);
        self.config.set_url(&normalized);
        self.client.set_base_url(&normalized);
        self.set_status(format!("Switched host to {}", normalized), false);
        self.reload_all();
    }

    pub fn reload_all(&mut self) {
        self.load_stats();
        self.load_pinned_memories();
        self.load_memories();
    }

    pub fn load_stats(&mut self) {
        match self.client.get_stats() {
            Ok(s) => self.stats = s,
            Err(e) => self.set_status(format!("Stats error: {}", e), true),
        }
    }

    pub fn load_pinned_memories(&mut self) {
        match self.client.list_memories(Some(true), None, 50) {
            Ok(pinned) => {
                self.pinned_memories = pinned;
            }
            Err(e) => {
                self.set_status(format!("Pinned memories error: {}", e), true);
            }
        }
    }

    pub fn total_sidebar_items(&self) -> usize {
        Scope::all().len() + self.pinned_memories.len()
    }

    pub fn get_sidebar_item(&self, idx: usize) -> ScopeItem {
        let scopes = Scope::all();
        if idx < scopes.len() {
            ScopeItem::Scope(scopes[idx])
        } else {
            let pinned_idx = idx - scopes.len();
            if let Some(m) = self.pinned_memories.get(pinned_idx) {
                ScopeItem::PinnedMemory(m.clone())
            } else {
                ScopeItem::Scope(Scope::All)
            }
        }
    }

    pub fn load_memories(&mut self) {
        if let Some(q) = &self.search_query {
            match self.client.search(q, "auto", 50) {
                Ok(results) => {
                    self.memories = results.into_iter().map(|r| r.memory).collect();
                }
                Err(e) => {
                    self.set_status(format!("Search error: {}", e), true);
                    self.memories.clear();
                }
            }
        } else {
            let scope_item = self.get_sidebar_item(self.sidebar_selected);
            match scope_item {
                ScopeItem::Scope(Scope::All) => {
                    match self.client.list_memories(None, None, 100) {
                        Ok(mut list) => {
                            sort_memories(&mut list);
                            self.memories = list;
                        }
                        Err(e) => {
                            self.set_status(format!("Memories load error: {}", e), true);
                            self.memories.clear();
                        }
                    }
                }
                ScopeItem::Scope(Scope::Pinned) => {
                    match self.client.list_memories(Some(true), None, 100) {
                        Ok(list) => {
                            self.memories = list;
                        }
                        Err(e) => {
                            self.set_status(format!("Pinned load error: {}", e), true);
                            self.memories.clear();
                        }
                    }
                }
                ScopeItem::Scope(Scope::Embedded) => {
                    match self.client.list_memories(None, None, 100) {
                        Ok(list) => {
                            let mut filtered: Vec<Memory> =
                                list.into_iter().filter(|m| m.embedded).collect();
                            sort_memories(&mut filtered);
                            self.memories = filtered;
                        }
                        Err(e) => {
                            self.set_status(format!("Embedded load error: {}", e), true);
                            self.memories.clear();
                        }
                    }
                }
                ScopeItem::Scope(Scope::Unembedded) => {
                    match self.client.list_memories(None, None, 100) {
                        Ok(list) => {
                            let mut filtered: Vec<Memory> =
                                list.into_iter().filter(|m| !m.embedded).collect();
                            sort_memories(&mut filtered);
                            self.memories = filtered;
                        }
                        Err(e) => {
                            self.set_status(format!("Unembedded load error: {}", e), true);
                            self.memories.clear();
                        }
                    }
                }
                ScopeItem::Scope(Scope::Recent) => {
                    match self.client.list_memories(None, None, 100) {
                        Ok(list) => {
                            self.memories = list;
                        }
                        Err(e) => {
                            self.set_status(format!("Recent load error: {}", e), true);
                            self.memories.clear();
                        }
                    }
                }
                ScopeItem::PinnedMemory(target) => {
                    self.memories = vec![target];
                }
            }
        }

        if self.memories_selected >= self.memories.len() {
            self.memories_selected = self.memories.len().saturating_sub(1);
        }

        self.load_current_memory_detail();
    }

    pub fn load_current_memory_detail(&mut self) {
        self.detail_scroll = 0;
        self.detail_selected_checklist = None;
        self.current_neighbors.clear();

        let mem = match self.memories.get(self.memories_selected) {
            Some(m) => m.clone(),
            None => {
                self.current_memory = None;
                self.checklist_items.clear();
                return;
            }
        };

        match self.client.get_memory(mem.id, true) {
            Ok(detail) => {
                self.checklist_items = parse_checklist_items(&detail.body);
                if !self.checklist_items.is_empty() {
                    self.detail_selected_checklist = Some(0);
                }
                self.current_memory = Some(detail);

                // Fetch nearest neighbors automatically
                if mem.embedded {
                    if let Ok(neighbors) = self.client.get_neighbors(mem.id, 5) {
                        self.current_neighbors = neighbors;
                    }
                }
            }
            Err(e) => {
                self.set_status(format!("Detail load error: {}", e), true);
                self.current_memory = None;
                self.checklist_items.clear();
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        self.clear_status();

        if let Some(m) = self.modal.clone() {
            self.handle_modal_key(m, key);
            return;
        }

        match key {
            KeyCode::Char('q') => {
                if self.search_query.is_some() {
                    self.search_query = None;
                    self.set_status("Filter cleared", false);
                    self.load_memories();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Esc => {
                if self.search_query.is_some() {
                    self.search_query = None;
                    self.set_status("Filter cleared", false);
                    self.load_memories();
                }
            }
            KeyCode::Tab => {
                self.focused_pane = self.focused_pane.next();
            }
            KeyCode::BackTab => {
                self.focused_pane = self.focused_pane.prev();
            }
            KeyCode::Char('1') => self.focused_pane = Pane::Sidebar,
            KeyCode::Char('2') => self.focused_pane = Pane::Memories,
            KeyCode::Char('3') => self.focused_pane = Pane::Detail,
            KeyCode::Char('h') | KeyCode::Left => self.focused_pane = self.focused_pane.prev(),
            KeyCode::Char('l') | KeyCode::Right => self.focused_pane = self.focused_pane.next(),
            KeyCode::Char('v') | KeyCode::Char('m') => {
                self.view_mode = self.view_mode.toggle();
                self.set_status(format!("View mode: {}", self.view_mode.label()), false);
            }
            KeyCode::Char('r') => {
                self.reload_all();
                self.set_status("Refreshed workspace", false);
            }
            KeyCode::Char('/') => {
                self.modal = Some(Modal::Search {
                    input: self.search_query.clone().unwrap_or_default(),
                });
            }
            KeyCode::Char('H') | KeyCode::Char('u') => {
                if self.config.allow_host_switcher {
                    self.modal = Some(Modal::HostSwitcher {
                        cursor: 0,
                        custom_input: String::new(),
                        entering_custom: false,
                    });
                } else {
                    self.set_status("Host switcher locked (MEMORABLE_REMOTE_URL set). Run with --allow-switch to enable.", true);
                }
            }
            KeyCode::Char('?') => {
                self.modal = Some(Modal::Help);
            }
            KeyCode::Char('a') => {
                if self.current_memory.is_some() {
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input: String::new(),
                        section_input: String::new(),
                        as_task: false,
                        focus_field: 0,
                    });
                } else {
                    self.set_status("No memory selected to append to", true);
                }
            }
            KeyCode::Char('n') => {
                self.modal = Some(Modal::MemoryForm {
                    id: None,
                    title_input: String::new(),
                    body_input: String::new(),
                    pinned: false,
                    focus_field: 0,
                });
            }
            KeyCode::Char('e') => {
                if let Some(mem) = &self.current_memory {
                    self.modal = Some(Modal::MemoryForm {
                        id: Some(mem.id),
                        title_input: mem.title.clone(),
                        body_input: mem.body.clone(),
                        pinned: mem.pinned,
                        focus_field: 0,
                    });
                }
            }
            KeyCode::Char('d') => {
                if let Some(mem) = self.memories.get(self.memories_selected) {
                    self.modal = Some(Modal::ConfirmDelete {
                        target: DeleteTarget::Memory(mem.clone()),
                    });
                }
            }
            KeyCode::Char('p') => {
                self.toggle_pin_current();
            }
            KeyCode::Char('V') => {
                self.generate_embedding_current();
            }
            KeyCode::Char('E') => {
                self.open_in_editor();
            }
            KeyCode::Char('c') | KeyCode::Char('Y') => {
                self.copy_memory_body();
            }
            KeyCode::Char('y') => {
                self.yank_active_item();
            }
            KeyCode::Char('A') => {
                if self.current_memory.is_some() {
                    self.modal = Some(Modal::Attachments {
                        cursor: 0,
                        upload_input: None,
                        dest_input: None,
                    });
                }
            }
            KeyCode::Char('j') | KeyCode::Down => match self.focused_pane {
                Pane::Sidebar => {
                    let total = self.total_sidebar_items();
                    if total > 0 && self.sidebar_selected + 1 < total {
                        self.sidebar_selected += 1;
                        self.load_memories();
                    }
                }
                Pane::Memories => {
                    if !self.memories.is_empty() && self.memories_selected + 1 < self.memories.len()
                    {
                        self.memories_selected += 1;
                        self.load_current_memory_detail();
                    }
                }
                Pane::Detail => {
                    if !self.checklist_items.is_empty() {
                        let cur = self.detail_selected_checklist.unwrap_or(0);
                        if cur + 1 < self.checklist_items.len() {
                            self.detail_selected_checklist = Some(cur + 1);
                        }
                    } else {
                        self.detail_scroll = self.detail_scroll.saturating_add(1);
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.focused_pane {
                Pane::Sidebar => {
                    if self.sidebar_selected > 0 {
                        self.sidebar_selected -= 1;
                        self.load_memories();
                    }
                }
                Pane::Memories => {
                    if self.memories_selected > 0 {
                        self.memories_selected -= 1;
                        self.load_current_memory_detail();
                    }
                }
                Pane::Detail => {
                    if !self.checklist_items.is_empty() {
                        let cur = self.detail_selected_checklist.unwrap_or(0);
                        if cur > 0 {
                            self.detail_selected_checklist = Some(cur - 1);
                        }
                    } else {
                        self.detail_scroll = self.detail_scroll.saturating_sub(1);
                    }
                }
            },
            KeyCode::Char('J') => match self.focused_pane {
                Pane::Memories => {
                    if self.is_pinned_scope() {
                        self.move_pinned_memory(true);
                    } else {
                        self.memories_selected =
                            (self.memories_selected + 5).min(self.memories.len().saturating_sub(1));
                        self.load_current_memory_detail();
                    }
                }
                Pane::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_add(5);
                }
                _ => {}
            },
            KeyCode::Char('K') => match self.focused_pane {
                Pane::Memories => {
                    if self.is_pinned_scope() {
                        self.move_pinned_memory(false);
                    } else {
                        self.memories_selected = self.memories_selected.saturating_sub(5);
                        self.load_current_memory_detail();
                    }
                }
                Pane::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(5);
                }
                _ => {}
            },
            KeyCode::PageDown => match self.focused_pane {
                Pane::Memories => {
                    self.memories_selected =
                        (self.memories_selected + 10).min(self.memories.len().saturating_sub(1));
                    self.load_current_memory_detail();
                }
                Pane::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_add(10);
                }
                _ => {}
            },
            KeyCode::PageUp => match self.focused_pane {
                Pane::Memories => {
                    self.memories_selected = self.memories_selected.saturating_sub(10);
                    self.load_current_memory_detail();
                }
                Pane::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(10);
                }
                _ => {}
            },
            KeyCode::Char('g') | KeyCode::Home => match self.focused_pane {
                Pane::Sidebar => {
                    self.sidebar_selected = 0;
                    self.load_memories();
                }
                Pane::Memories => {
                    self.memories_selected = 0;
                    self.load_current_memory_detail();
                }
                Pane::Detail => {
                    self.detail_scroll = 0;
                    if !self.checklist_items.is_empty() {
                        self.detail_selected_checklist = Some(0);
                    }
                }
            },
            KeyCode::Char('G') | KeyCode::End => match self.focused_pane {
                Pane::Sidebar => {
                    let total = self.total_sidebar_items();
                    if total > 0 {
                        self.sidebar_selected = total - 1;
                        self.load_memories();
                    }
                }
                Pane::Memories => {
                    if !self.memories.is_empty() {
                        self.memories_selected = self.memories.len() - 1;
                        self.load_current_memory_detail();
                    }
                }
                Pane::Detail => {
                    self.detail_scroll = 100;
                    if !self.checklist_items.is_empty() {
                        self.detail_selected_checklist = Some(self.checklist_items.len() - 1);
                    }
                }
            },
            KeyCode::Enter => match self.focused_pane {
                Pane::Sidebar => {
                    self.focused_pane = Pane::Memories;
                }
                Pane::Memories => {
                    self.focused_pane = Pane::Detail;
                }
                Pane::Detail => {
                    self.toggle_active_checklist();
                }
            },
            KeyCode::Char(' ') if self.focused_pane == Pane::Detail => {
                self.toggle_active_checklist();
            }
            _ => {}
        }
    }

    fn is_pinned_scope(&self) -> bool {
        matches!(
            self.get_sidebar_item(self.sidebar_selected),
            ScopeItem::Scope(Scope::Pinned)
        )
    }

    fn move_pinned_memory(&mut self, down: bool) {
        if self.memories.len() < 2 {
            return;
        }
        let cur = self.memories_selected;
        let target = if down {
            if cur + 1 < self.memories.len() {
                cur + 1
            } else {
                return;
            }
        } else if cur > 0 {
            cur - 1
        } else {
            return;
        };

        self.memories.swap(cur, target);
        self.memories_selected = target;

        let ids: Vec<i64> = self.memories.iter().map(|m| m.id).collect();
        if let Err(e) = self.client.reorder_pinned(&ids) {
            self.set_status(format!("Reorder error: {}", e), true);
        } else {
            self.set_status("Pinned order updated", false);
            self.load_pinned_memories();
        }
    }

    pub fn toggle_pin_current(&mut self) {
        let mem = match self.memories.get(self.memories_selected) {
            Some(m) => m.clone(),
            None => return,
        };

        match self.client.toggle_pin(mem.id) {
            Ok(pinned) => {
                let status_msg = if pinned {
                    format!("Pinned '{}'", mem.title)
                } else {
                    format!("Unpinned '{}'", mem.title)
                };
                self.set_status(status_msg, false);
                self.reload_all();
            }
            Err(e) => {
                self.set_status(format!("Pin toggle error: {}", e), true);
            }
        }
    }

    pub fn generate_embedding_current(&mut self) {
        let mem = match self.memories.get(self.memories_selected) {
            Some(m) => m.clone(),
            None => return,
        };

        match self.client.generate_embedding(mem.id) {
            Ok(()) => {
                self.set_status(
                    format!("Vector embedding generated for '{}'", mem.title),
                    false,
                );
                self.reload_all();
            }
            Err(e) => {
                self.set_status(format!("Embedding error: {}", e), true);
            }
        }
    }

    pub fn copy_memory_body(&mut self) {
        if let Some(detail) = &self.current_memory {
            if let Some(cb) = &mut self.clipboard {
                if let Err(e) = cb.set_text(&detail.body) {
                    self.set_status(format!("Clipboard error: {}", e), true);
                } else {
                    self.set_status(
                        format!("Copied body of '{}' to clipboard", detail.title),
                        false,
                    );
                }
            } else {
                self.set_status("Clipboard not available", true);
            }
        }
    }

    pub fn yank_active_item(&mut self) {
        if self.focused_pane == Pane::Detail {
            if let Some(idx) = self.detail_selected_checklist {
                if let Some(item) = self.checklist_items.get(idx) {
                    if let Some(cb) = &mut self.clipboard {
                        if let Err(e) = cb.set_text(&item.text) {
                            self.set_status(format!("Clipboard error: {}", e), true);
                        } else {
                            self.set_status(format!("Yanked task: \"{}\"", item.text), false);
                        }
                    }
                    return;
                }
            }
        }

        if let Some(mem) = self.memories.get(self.memories_selected) {
            if let Some(cb) = &mut self.clipboard {
                if let Err(e) = cb.set_text(&mem.title) {
                    self.set_status(format!("Clipboard error: {}", e), true);
                } else {
                    self.set_status(format!("Yanked title: \"{}\"", mem.title), false);
                }
            }
        }
    }

    pub fn open_in_editor(&mut self) {
        let detail = match &self.current_memory {
            Some(d) => d.clone(),
            None => return,
        };

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("memorable_memory_{}.md", detail.id));

        if let Err(e) = std::fs::write(&file_path, &detail.body) {
            self.set_status(format!("Failed to write temp file: {}", e), true);
            return;
        }

        self.pending_editor = Some((detail.id, file_path));
    }

    pub fn complete_external_editor(&mut self) -> Result<()> {
        if let Some((id, path)) = self.pending_editor.take() {
            self.finish_editor(id, path);
        }
        Ok(())
    }

    pub fn finish_editor(&mut self, memory_id: i64, path: PathBuf) {
        self.pending_editor = None;
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                self.set_status(format!("Failed to read edited file: {}", e), true);
                let _ = std::fs::remove_file(path);
                return;
            }
        };

        let _ = std::fs::remove_file(path);

        if let Some(cur) = &self.current_memory {
            if cur.id == memory_id && cur.body == content {
                self.set_status("No changes made in editor", false);
                return;
            }
        }

        let title = self
            .current_memory
            .as_ref()
            .map(|d| d.title.clone())
            .unwrap_or_else(|| "Untitled".to_string());

        let input = MemoryInput {
            title,
            body: content,
            pinned: None,
        };

        match self.client.update_memory(memory_id, &input) {
            Ok(_) => {
                self.set_status("Memory updated from $EDITOR", false);
                self.load_memories();
            }
            Err(e) => {
                self.set_status(format!("Failed to save memory: {}", e), true);
            }
        }
    }

    pub fn toggle_active_checklist(&mut self) {
        let selected_idx = match self.detail_selected_checklist {
            Some(i) => i,
            None => return,
        };

        let cur = match &self.current_memory {
            Some(d) => d.clone(),
            None => return,
        };

        if let Some((new_body, _item)) = toggle_checklist_item_by_index(&cur.body, selected_idx) {
            let input = MemoryInput {
                title: cur.title.clone(),
                body: new_body.clone(),
                pinned: None,
            };

            match self.client.update_memory(cur.id, &input) {
                Ok(_) => {
                    self.checklist_items = parse_checklist_items(&new_body);
                    if let Some(mem) = self.current_memory.as_mut() {
                        mem.body = new_body;
                    }
                    if let Some(m) = self.memories.get_mut(self.memories_selected) {
                        m.body = input.body;
                    }
                }
                Err(e) => {
                    self.set_status(format!("Toggle error: {}", e), true);
                }
            }
        }
    }

    fn handle_modal_key(&mut self, modal: Modal, key: KeyCode) {
        match modal {
            Modal::Help => {
                if matches!(key, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
                    self.modal = None;
                }
            }
            Modal::Search { mut input } => match key {
                KeyCode::Esc => {
                    self.modal = None;
                }
                KeyCode::Enter => {
                    self.modal = None;
                    if input.trim().is_empty() {
                        self.search_query = None;
                        self.set_status("Filter cleared", false);
                    } else {
                        self.search_query = Some(input.trim().to_string());
                        self.set_status(format!("Searching: \"{}\"", input.trim()), false);
                    }
                    self.load_memories();
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::Search { input });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::Search { input });
                }
                _ => {}
            },
            Modal::HostSwitcher {
                mut cursor,
                mut custom_input,
                mut entering_custom,
            } => {
                let total_options = self.preset_hosts.len() + 1; // presets + custom entry
                match key {
                    KeyCode::Esc => {
                        self.modal = None;
                    }
                    KeyCode::Char('j') | KeyCode::Down if !entering_custom => {
                        if cursor + 1 < total_options {
                            cursor += 1;
                            self.modal = Some(Modal::HostSwitcher {
                                cursor,
                                custom_input,
                                entering_custom,
                            });
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up if !entering_custom => {
                        if cursor > 0 {
                            cursor -= 1;
                            self.modal = Some(Modal::HostSwitcher {
                                cursor,
                                custom_input,
                                entering_custom,
                            });
                        }
                    }
                    KeyCode::Enter => {
                        if cursor < self.preset_hosts.len() {
                            let selected = self.preset_hosts[cursor].clone();
                            self.modal = None;
                            self.switch_host(&selected);
                        } else if entering_custom {
                            if !custom_input.trim().is_empty() {
                                let selected = custom_input.trim().to_string();
                                self.modal = None;
                                self.switch_host(&selected);
                            }
                        } else {
                            entering_custom = true;
                            self.modal = Some(Modal::HostSwitcher {
                                cursor,
                                custom_input,
                                entering_custom,
                            });
                        }
                    }
                    KeyCode::Backspace if entering_custom => {
                        custom_input.pop();
                        self.modal = Some(Modal::HostSwitcher {
                            cursor,
                            custom_input,
                            entering_custom,
                        });
                    }
                    KeyCode::Char(c) if entering_custom => {
                        custom_input.push(c);
                        self.modal = Some(Modal::HostSwitcher {
                            cursor,
                            custom_input,
                            entering_custom,
                        });
                    }
                    _ => {}
                }
            }
            Modal::QuickAppendMemory {
                mut content_input,
                mut section_input,
                mut as_task,
                mut focus_field,
            } => match key {
                KeyCode::Esc => {
                    self.modal = None;
                }
                KeyCode::Tab => {
                    focus_field = (focus_field + 1) % 3;
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input,
                        section_input,
                        as_task,
                        focus_field,
                    });
                }
                KeyCode::BackTab => {
                    focus_field = (focus_field + 2) % 3;
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input,
                        section_input,
                        as_task,
                        focus_field,
                    });
                }
                KeyCode::Enter => {
                    if focus_field == 2 {
                        as_task = !as_task;
                        self.modal = Some(Modal::QuickAppendMemory {
                            content_input,
                            section_input,
                            as_task,
                            focus_field,
                        });
                    } else if !content_input.trim().is_empty() {
                        self.modal = None;
                        if let Some(cur) = &self.current_memory {
                            let sec_opt = if section_input.trim().is_empty() {
                                None
                            } else {
                                Some(section_input.trim())
                            };
                            let updated_body =
                                append_to_memory(&cur.body, &content_input, sec_opt, as_task);
                            let input = MemoryInput {
                                title: cur.title.clone(),
                                body: updated_body,
                                pinned: None,
                            };
                            match self.client.update_memory(cur.id, &input) {
                                Ok(_) => {
                                    self.set_status("Content appended to memory", false);
                                    self.load_memories();
                                }
                                Err(e) => {
                                    self.set_status(format!("Append error: {}", e), true);
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') if focus_field == 2 => {
                    as_task = !as_task;
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input,
                        section_input,
                        as_task,
                        focus_field,
                    });
                }
                KeyCode::Backspace => {
                    if focus_field == 0 {
                        content_input.pop();
                    } else if focus_field == 1 {
                        section_input.pop();
                    }
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input,
                        section_input,
                        as_task,
                        focus_field,
                    });
                }
                KeyCode::Char(c) => {
                    if focus_field == 0 {
                        content_input.push(c);
                    } else if focus_field == 1 {
                        section_input.push(c);
                    }
                    self.modal = Some(Modal::QuickAppendMemory {
                        content_input,
                        section_input,
                        as_task,
                        focus_field,
                    });
                }
                _ => {}
            },
            Modal::MemoryForm {
                id,
                mut title_input,
                mut body_input,
                mut pinned,
                mut focus_field,
            } => match key {
                KeyCode::Esc => {
                    self.modal = None;
                }
                KeyCode::Tab => {
                    focus_field = (focus_field + 1) % 3;
                    self.modal = Some(Modal::MemoryForm {
                        id,
                        title_input,
                        body_input,
                        pinned,
                        focus_field,
                    });
                }
                KeyCode::BackTab => {
                    focus_field = (focus_field + 2) % 3;
                    self.modal = Some(Modal::MemoryForm {
                        id,
                        title_input,
                        body_input,
                        pinned,
                        focus_field,
                    });
                }
                KeyCode::Char(' ') if focus_field == 2 => {
                    pinned = !pinned;
                    self.modal = Some(Modal::MemoryForm {
                        id,
                        title_input,
                        body_input,
                        pinned,
                        focus_field,
                    });
                }
                KeyCode::Enter => {
                    if focus_field == 2 {
                        pinned = !pinned;
                        self.modal = Some(Modal::MemoryForm {
                            id,
                            title_input,
                            body_input,
                            pinned,
                            focus_field,
                        });
                    } else if !title_input.trim().is_empty() {
                        self.modal = None;
                        let input = MemoryInput {
                            title: title_input.trim().to_string(),
                            body: body_input,
                            pinned: Some(pinned),
                        };

                        if let Some(mem_id) = id {
                            match self.client.update_memory(mem_id, &input) {
                                Ok(_) => {
                                    self.set_status("Memory updated", false);
                                    self.reload_all();
                                }
                                Err(e) => {
                                    self.set_status(format!("Update error: {}", e), true);
                                }
                            }
                        } else {
                            match self.client.create_memory(&input) {
                                Ok(new_mem) => {
                                    self.set_status(
                                        format!("Created memory '{}'", new_mem.title),
                                        false,
                                    );
                                    self.reload_all();
                                }
                                Err(e) => {
                                    self.set_status(format!("Create error: {}", e), true);
                                }
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    if focus_field == 0 {
                        title_input.pop();
                    } else if focus_field == 1 {
                        body_input.pop();
                    }
                    self.modal = Some(Modal::MemoryForm {
                        id,
                        title_input,
                        body_input,
                        pinned,
                        focus_field,
                    });
                }
                KeyCode::Char(c) => {
                    if focus_field == 0 {
                        title_input.push(c);
                    } else if focus_field == 1 {
                        body_input.push(c);
                    }
                    self.modal = Some(Modal::MemoryForm {
                        id,
                        title_input,
                        body_input,
                        pinned,
                        focus_field,
                    });
                }
                _ => {}
            },
            Modal::ConfirmDelete { target } => match key {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.modal = None;
                    match target {
                        DeleteTarget::Memory(mem) => match self.client.delete_memory(mem.id) {
                            Ok(_) => {
                                self.set_status(format!("Deleted memory '{}'", mem.title), false);
                                self.reload_all();
                            }
                            Err(e) => {
                                self.set_status(format!("Delete error: {}", e), true);
                            }
                        },
                        DeleteTarget::Attachment {
                            memory_id,
                            attachment_id,
                            filename,
                        } => {
                            match self.client.delete_attachment(memory_id, attachment_id) {
                                Ok(_) => {
                                    self.set_status(
                                        format!("Removed attachment '{}'", filename),
                                        false,
                                    );
                                    self.load_current_memory_detail();
                                }
                                Err(e) => {
                                    self.set_status(format!("Delete error: {}", e), true);
                                }
                            }
                        }
                    }
                }
                _ => {
                    self.modal = None;
                }
            },
            Modal::Attachments {
                mut cursor,
                upload_input,
                dest_input,
            } => {
                let attachments = self
                    .current_memory
                    .as_ref()
                    .map(|d| d.attachments.clone())
                    .unwrap_or_default();

                if let Some(mut inp) = upload_input.clone() {
                    match key {
                        KeyCode::Esc => {
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                        KeyCode::Enter => {
                            if !inp.trim().is_empty() {
                                let path = PathBuf::from(inp.trim());
                                if let Some(mem) = &self.current_memory {
                                    match self.client.upload_attachment(mem.id, &path) {
                                        Ok(_) => {
                                            self.set_status("Attachment uploaded", false);
                                            self.load_current_memory_detail();
                                        }
                                        Err(e) => {
                                            self.set_status(format!("Upload error: {}", e), true);
                                        }
                                    }
                                }
                            }
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                        KeyCode::Backspace => {
                            inp.pop();
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: Some(inp),
                                dest_input: None,
                            });
                        }
                        KeyCode::Char(c) => {
                            inp.push(c);
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: Some(inp),
                                dest_input: None,
                            });
                        }
                        _ => {}
                    }
                    return;
                }

                if let Some(mut inp) = dest_input.clone() {
                    match key {
                        KeyCode::Esc => {
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                        KeyCode::Enter => {
                            if !inp.trim().is_empty() {
                                if let Some(att) = attachments.get(cursor) {
                                    if let Some(url) = &att.url {
                                        let path = PathBuf::from(inp.trim());
                                        match self.client.download_attachment(url, &path) {
                                            Ok(_) => {
                                                self.set_status(
                                                    format!("Downloaded to {}", path.display()),
                                                    false,
                                                );
                                            }
                                            Err(e) => {
                                                self.set_status(
                                                    format!("Download error: {}", e),
                                                    true,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                        KeyCode::Backspace => {
                            inp.pop();
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: Some(inp),
                            });
                        }
                        KeyCode::Char(c) => {
                            inp.push(c);
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: Some(inp),
                            });
                        }
                        _ => {}
                    }
                    return;
                }

                match key {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.modal = None;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if !attachments.is_empty() && cursor + 1 < attachments.len() {
                            cursor += 1;
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if cursor > 0 {
                            cursor -= 1;
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: None,
                            });
                        }
                    }
                    KeyCode::Char('u') => {
                        self.modal = Some(Modal::Attachments {
                            cursor,
                            upload_input: Some(String::new()),
                            dest_input: None,
                        });
                    }
                    KeyCode::Char('s') => {
                        if let Some(att) = attachments.get(cursor) {
                            self.modal = Some(Modal::Attachments {
                                cursor,
                                upload_input: None,
                                dest_input: Some(att.filename.clone()),
                            });
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(att) = attachments.get(cursor) {
                            if let Some(mem) = &self.current_memory {
                                self.modal = Some(Modal::ConfirmDelete {
                                    target: DeleteTarget::Attachment {
                                        memory_id: mem.id,
                                        attachment_id: att.id,
                                        filename: att.filename.clone(),
                                    },
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
