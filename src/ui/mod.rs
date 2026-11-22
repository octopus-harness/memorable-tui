pub mod detail;
pub mod memory_list;
pub mod modals;
pub mod sidebar;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Pane};

pub const COLOR_VIOLET: Color = Color::Rgb(139, 92, 246);
pub const COLOR_EMERALD: Color = Color::Rgb(34, 197, 94);
pub const COLOR_AMBER: Color = Color::Rgb(245, 158, 11);
pub const COLOR_SLATE: Color = Color::Rgb(148, 163, 184);
pub const COLOR_BORDER: Color = Color::Rgb(71, 85, 105);

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, app, chunks[0]);

    // 3-Pane Body
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Percentage(38),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    sidebar::render(f, app, body_chunks[0]);
    memory_list::render(f, app, body_chunks[1]);
    detail::render(f, app, body_chunks[2]);

    render_footer(f, app, chunks[2]);

    // Modals overlay
    modals::render(f, app);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let host_badge = format!(" [Host: {}] ", app.config.base_url);
    let stats = &app.stats;
    let stats_text = format!(
        "Memories: {} (★ {}) · Attachments: {}",
        stats.total_memories, stats.pinned_memories, stats.total_attachments
    );

    let mut spans = vec![
        Span::styled(
            " 🧠 MEMORABLE ",
            Style::default()
                .fg(Color::Black)
                .bg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            host_badge,
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if app.config.allow_host_switcher {
        spans.push(Span::styled(
            " (H to switch) ",
            Style::default().fg(COLOR_SLATE),
        ));
    }

    spans.push(Span::styled(
        format!(" · {}", stats_text),
        Style::default().fg(COLOR_SLATE),
    ));

    spans.push(Span::styled(
        format!(" · [View: {}]", app.view_mode.label()),
        Style::default().fg(COLOR_EMERALD),
    ));

    if let Some(query) = &app.search_query {
        spans.push(Span::styled(
            format!(" · [Filter: \"{}\"] (Esc to clear)", query),
            Style::default()
                .fg(COLOR_AMBER)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER)),
    );
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_span = if let Some((msg, is_error)) = &app.status {
        let color = if *is_error { Color::Red } else { COLOR_EMERALD };
        Span::styled(
            format!(" ● {} ", msg),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )
    } else {
        let focus_hint = match app.focused_pane {
            Pane::Sidebar => {
                "Pane 1 (Scopes & Pinned): j/k: nav | Enter: view memories"
            }
            Pane::Memories => {
                "Pane 2 (Memories): j/k: nav | Enter: view | p: pin | V: embed | n: new | e: edit | d: del"
            }
            Pane::Detail => {
                "Pane 3 (Detail): j/k: checklist | Space: toggle | v: view | a: append | V: embed | E: $EDITOR"
            }
        };
        Span::styled(
            format!(" {} ", focus_hint),
            Style::default().fg(COLOR_SLATE),
        )
    };

    let key_hints_str = if app.config.allow_host_switcher {
        "1/2/3: panes | v: view | /: search | H: host | ?: help | q: quit "
    } else {
        "1/2/3: panes | v: view | /: search | ?: help | q: quit "
    };

    let key_hints = Span::styled(key_hints_str, Style::default().fg(COLOR_SLATE));

    let total_width = area.width as usize;
    let left_len = status_span.content.len();
    let right_len = key_hints.content.len();
    let pad = total_width.saturating_sub(left_len + right_len);

    let line = Line::from(vec![status_span, Span::raw(" ".repeat(pad)), key_hints]);

    f.render_widget(Paragraph::new(line), area);
}
