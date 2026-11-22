use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, DeleteTarget, Modal};
use crate::format::format_bytes;
use crate::ui::{COLOR_AMBER, COLOR_BORDER, COLOR_EMERALD, COLOR_SLATE, COLOR_VIOLET};

pub fn render(f: &mut Frame, app: &App) {
    let modal = match &app.modal {
        Some(m) => m,
        None => return,
    };

    match modal {
        Modal::HostSwitcher {
            cursor,
            custom_input,
            entering_custom,
        } => render_host_switcher(f, app, *cursor, custom_input, *entering_custom),
        Modal::Search { input } => render_search(f, input),
        Modal::QuickAppendMemory {
            content_input,
            section_input,
            as_task,
            focus_field,
        } => render_quick_append(f, content_input, section_input, *as_task, *focus_field),
        Modal::MemoryForm {
            id,
            title_input,
            body_input,
            pinned,
            focus_field,
        } => render_memory_form(f, id.is_some(), title_input, body_input, *pinned, *focus_field),
        Modal::ConfirmDelete { target } => render_confirm_delete(f, target),
        Modal::Attachments {
            cursor,
            upload_input,
            dest_input,
        } => render_attachments(
            f,
            app,
            *cursor,
            upload_input.as_deref(),
            dest_input.as_deref(),
        ),
        Modal::Help => render_help(f, app.config.allow_host_switcher),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_host_switcher(
    f: &mut Frame,
    app: &App,
    cursor: usize,
    custom_input: &str,
    entering_custom: bool,
) {
    let area = centered_rect(60, 45, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Switch Target Memorable Host ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut items = Vec::new();
    for (idx, host) in app.preset_hosts.iter().enumerate() {
        let is_selected = idx == cursor && !entering_custom;
        let is_current = host == &app.config.base_url;

        let marker = if is_selected { "▶ " } else { "  " };
        let current_badge = if is_current { " [Active]" } else { "" };

        let style = if is_selected {
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(host, style),
            Span::styled(
                current_badge,
                Style::default()
                    .fg(COLOR_EMERALD)
                    .add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    // Custom Host Entry
    let custom_idx = app.preset_hosts.len();
    let is_selected = cursor == custom_idx;
    let marker = if is_selected { "▶ " } else { "  " };
    let prompt = if entering_custom {
        format!("Custom: {}█", custom_input)
    } else {
        format!("Enter Custom URL... (current: {})", custom_input)
    };

    let style = if is_selected {
        Style::default()
            .fg(COLOR_EMERALD)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SLATE)
    };

    items.push(ListItem::new(Line::from(vec![
        Span::styled(marker, style),
        Span::styled(prompt, style),
    ])));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    f.render_widget(List::new(items), layout[0]);

    let footer_text = if entering_custom {
        "Type URL and press Enter to connect | Esc to cancel"
    } else {
        "j/k: select preset | Enter: switch | Esc: cancel"
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(footer, layout[1]);
}

fn render_search(f: &mut Frame, input: &str) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Search & Filter Memories (Vector Auto Mode) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let label = Paragraph::new(Line::from(Span::styled(
        "Enter query (automatically runs 1024-dim semantic or keyword search):",
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(label, layout[0]);

    let input_para = Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(COLOR_VIOLET)),
        Span::styled(
            input,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("█", Style::default().fg(COLOR_EMERALD)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER)),
    );
    f.render_widget(input_para, layout[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "Enter: search | Esc: cancel / clear filter",
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(footer, layout[2]);
}

fn render_quick_append(
    f: &mut Frame,
    content: &str,
    section: &str,
    as_task: bool,
    focus: usize,
) {
    let area = centered_rect(65, 38, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Quick Append to Memory ('a') ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(inner);

    let content_border = if focus == 0 {
        COLOR_VIOLET
    } else {
        COLOR_BORDER
    };
    let content_block = Block::default()
        .title(" Content / Task Text ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(content_border));

    let cursor = if focus == 0 { "█" } else { "" };
    let content_line = Line::from(vec![
        Span::styled(content, Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(COLOR_EMERALD)),
    ]);
    f.render_widget(Paragraph::new(content_line).block(content_block), layout[0]);

    let section_border = if focus == 1 {
        COLOR_VIOLET
    } else {
        COLOR_BORDER
    };
    let section_block = Block::default()
        .title(" Optional Section Header (e.g. 'Action Items' or '## Decision') ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(section_border));

    let cursor = if focus == 1 { "█" } else { "" };
    let sec_line = Line::from(vec![
        Span::styled(section, Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(COLOR_EMERALD)),
    ]);
    f.render_widget(Paragraph::new(sec_line).block(section_block), layout[1]);

    let task_check = if as_task { "[x]" } else { "[ ]" };
    let task_color = if as_task {
        COLOR_EMERALD
    } else {
        COLOR_SLATE
    };
    let task_style = if focus == 2 {
        Style::default()
            .fg(COLOR_VIOLET)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SLATE)
    };

    let task_line = Line::from(vec![
        Span::styled(format!("  {} ", task_check), Style::default().fg(task_color)),
        Span::styled("Format as Markdown checklist task (- [ ])", task_style),
        Span::styled(
            " (Space to toggle)",
            Style::default().fg(COLOR_SLATE),
        ),
    ]);
    f.render_widget(Paragraph::new(task_line), layout[2]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "Tab: next field | Enter: append to memory | Esc: cancel",
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(footer, layout[3]);
}

fn render_memory_form(
    f: &mut Frame,
    is_edit: bool,
    title: &str,
    body: &str,
    pinned: bool,
    focus: usize,
) {
    let area = centered_rect(65, 45, f.area());
    f.render_widget(Clear, area);

    let modal_title = if is_edit {
        " Edit Memory Metadata "
    } else {
        " Create New Memory ('n') "
    };

    let block = Block::default()
        .title(modal_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(inner);

    let title_border = if focus == 0 {
        COLOR_VIOLET
    } else {
        COLOR_BORDER
    };
    let title_cursor = if focus == 0 { "█" } else { "" };
    let title_para = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().fg(Color::White)),
        Span::styled(title_cursor, Style::default().fg(COLOR_EMERALD)),
    ]))
    .block(
        Block::default()
            .title(" Memory Title (Required) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(title_border)),
    );
    f.render_widget(title_para, layout[0]);

    let body_border = if focus == 1 {
        COLOR_VIOLET
    } else {
        COLOR_BORDER
    };
    let body_cursor = if focus == 1 { "█" } else { "" };
    let body_para = Paragraph::new(Line::from(vec![
        Span::styled(body, Style::default().fg(Color::White)),
        Span::styled(body_cursor, Style::default().fg(COLOR_EMERALD)),
    ]))
    .block(
        Block::default()
            .title(" Markdown Body ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(body_border)),
    );
    f.render_widget(body_para, layout[1]);

    let pin_check = if pinned { "[x]" } else { "[ ]" };
    let pin_color = if pinned { COLOR_AMBER } else { COLOR_SLATE };
    let pin_style = if focus == 2 {
        Style::default()
            .fg(COLOR_VIOLET)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SLATE)
    };

    let pin_line = Line::from(vec![
        Span::styled(format!("  {} ", pin_check), Style::default().fg(pin_color)),
        Span::styled("Pin to top of workspace (★)", pin_style),
        Span::styled(" (Space to toggle)", Style::default().fg(COLOR_SLATE)),
    ]);
    f.render_widget(Paragraph::new(pin_line), layout[2]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "Tab: cycle fields | Enter: save memory | Esc: cancel",
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(footer, layout[3]);
}

fn render_confirm_delete(f: &mut Frame, target: &DeleteTarget) {
    let area = centered_rect(55, 25, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm Deletion ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let prompt = match target {
        DeleteTarget::Memory(mem) => {
            format!("Permanently delete memory '{}'?", mem.title)
        }
        DeleteTarget::Attachment { filename, .. } => {
            format!("Remove attachment '{}'?", filename)
        }
    };

    let lines = vec![
        Line::from(Span::styled(
            prompt,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default().fg(COLOR_SLATE),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(COLOR_SLATE)),
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(COLOR_SLATE)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to confirm, or any other key to cancel", Style::default().fg(COLOR_SLATE)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_attachments(
    f: &mut Frame,
    app: &App,
    cursor: usize,
    upload_input: Option<&str>,
    dest_input: Option<&str>,
) {
    let area = centered_rect(70, 50, f.area());
    f.render_widget(Clear, area);

    let mem_title = app
        .current_memory
        .as_ref()
        .map(|d| d.title.as_str())
        .unwrap_or("Memory");

    let block = Block::default()
        .title(format!(" Context Attachments: {} ", mem_title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(inp) = upload_input {
        let input_area = centered_rect(80, 30, inner);
        let upload_block = Block::default()
            .title(" Upload Attachment (Path to file) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_EMERALD));

        let p = Paragraph::new(Line::from(vec![
            Span::styled(inp, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(COLOR_EMERALD)),
        ]))
        .block(upload_block);

        f.render_widget(p, input_area);
        return;
    }

    if let Some(inp) = dest_input {
        let input_area = centered_rect(80, 30, inner);
        let save_block = Block::default()
            .title(" Save Attachment to Path ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_EMERALD));

        let p = Paragraph::new(Line::from(vec![
            Span::styled(inp, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(COLOR_EMERALD)),
        ]))
        .block(save_block);

        f.render_widget(p, input_area);
        return;
    }

    let attachments = app
        .current_memory
        .as_ref()
        .map(|d| d.attachments.as_slice())
        .unwrap_or(&[]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let mut items = Vec::new();
    if attachments.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  (no attachments on this memory — press 'u' to upload a context file)",
            Style::default().fg(COLOR_SLATE),
        ))));
    } else {
        for (idx, att) in attachments.iter().enumerate() {
            let is_selected = idx == cursor;
            let marker = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let size_str = att
                .byte_size
                .map(|b| format_bytes(b as u64))
                .unwrap_or_else(|| "0 B".into());

            let line = Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("📎 {:<32}", att.filename), style),
                Span::styled(
                    format!(" {:>10}", size_str),
                    Style::default().fg(COLOR_EMERALD),
                ),
            ]);
            items.push(ListItem::new(line));
        }
    }

    f.render_widget(List::new(items), layout[0]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "Keys: j/k: select | u: upload file | s: save to disk | d: delete | Esc: close",
        Style::default().fg(COLOR_SLATE),
    )));
    f.render_widget(footer, layout[1]);
}

fn render_help(f: &mut Frame, allow_switch: bool) {
    let area = centered_rect(76, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Memorable-TUI Keyboard Shortcuts & Architecture ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_VIOLET));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let host_line = if allow_switch {
        Line::from("  H / u          Switch Host URL presets or enter custom URL")
    } else {
        Line::from(
            "  H / u          [Locked by MEMORABLE_REMOTE_URL] (pass --allow-switch to enable)",
        )
    };

    let lines = vec![
        Line::from(Span::styled(
            "Navigation & Focus",
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  1, 2, 3        Direct focus to pane 1 (Sidebar), 2 (Memories), 3 (Detail)"),
        Line::from("  Tab / BackTab  Cycle focus forward / backward between panes"),
        Line::from("  h / l          Move focus left / right"),
        Line::from("  j / k          Move selection down / up in focused pane"),
        Line::from("  Enter          Enter pane 2 from sidebar, or pane 3 from memories list"),
        Line::from(""),
        Line::from(Span::styled(
            "Memory Detail, Markdown & Checklists (Pane 3)",
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  v / m          Toggle between Styled Preview and Raw Markdown (line numbers)"),
        Line::from("  j / k          Navigate checklist items (or scroll document when no tasks)"),
        Line::from("  Space / Enter  Toggle active checklist item [ ] ⇄ [x]"),
        Line::from("  a              Quick-append Markdown section, log entry, or task to memory"),
        Line::from("  V              Trigger 1024-dim vector embedding generation (pgvector)"),
        Line::from("  E              Open memory Markdown body in external $EDITOR (nvim/vim/nano)"),
        Line::from("  J / K          Scroll detail document view down / up by 5 lines"),
        Line::from("  g / G          Jump to top / bottom of document"),
        Line::from("  c / Y          Copy entire Markdown body to system clipboard"),
        Line::from("  y              Yank active task text (or memory title) to clipboard"),
        Line::from(""),
        Line::from(Span::styled(
            "Workspace Management & Modals",
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        )),
        host_line,
        Line::from("  /              Real-time vector & keyword search (Esc to clear filter)"),
        Line::from("  n              Create New Memory in workspace"),
        Line::from("  e              Edit title & metadata of selected memory"),
        Line::from("  d              Delete selected memory or context attachment"),
        Line::from("  p              Toggle pinned status of selected memory"),
        Line::from("  K / J          Reorder pinned memories up / down (in Pinned scope)"),
        Line::from("  A              Open context attachments manager (upload, download, delete)"),
        Line::from("  r              Refresh all data and nearest neighbors from Memorable server"),
        Line::from("  ?              Open this help dialog"),
        Line::from("  q              Quit application (or Esc to close dialogs)"),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
