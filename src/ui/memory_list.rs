use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, Pane, ScopeItem};
use crate::checklist::checklist_progress;
use crate::format::date_short;
use crate::ui::{COLOR_AMBER, COLOR_BORDER, COLOR_EMERALD, COLOR_SLATE, COLOR_VIOLET};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Memories;
    let border_color = if is_focused {
        COLOR_VIOLET
    } else {
        COLOR_BORDER
    };

    let title_style = if is_focused {
        Style::default()
            .fg(COLOR_VIOLET)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_SLATE)
    };

    let scope_name = match app.get_sidebar_item(app.sidebar_selected) {
        ScopeItem::Scope(s) => s.label().to_string(),
        ScopeItem::PinnedMemory(m) => format!("★ {}", m.title),
    };

    let title_text = format!(" [2] Memories ({} · {}) ", scope_name, app.memories.len());

    let mut items = Vec::new();

    if app.memories.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  (no memories found — press 'n' to create one)",
            Style::default().fg(COLOR_SLATE),
        ))));
    } else {
        for (idx, mem) in app.memories.iter().enumerate() {
            let is_selected = app.memories_selected == idx;
            let cursor = if is_selected { "▶ " } else { "  " };

            let star = if mem.pinned { "★ " } else { "  " };
            let star_style = if mem.pinned {
                Style::default().fg(COLOR_AMBER)
            } else {
                Style::default().fg(COLOR_BORDER)
            };

            let item_title_style = if is_selected {
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Vector embedded badge
            let emb_span = if mem.embedded {
                Span::styled(
                    "[EMB] ",
                    Style::default()
                        .fg(COLOR_EMERALD)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            };

            // Checklist progress
            let (done, total) = checklist_progress(&mem.body);
            let progress_span = if total > 0 {
                let color = if done == total {
                    COLOR_EMERALD
                } else if done > 0 {
                    COLOR_AMBER
                } else {
                    COLOR_SLATE
                };
                Span::styled(format!(" [{}/{}]", done, total), Style::default().fg(color))
            } else {
                Span::raw("")
            };

            // Attachments indicator
            let att_span = if mem.attachments_count > 0 {
                Span::styled(
                    format!(" 📎{}", mem.attachments_count),
                    Style::default().fg(Color::Cyan),
                )
            } else {
                Span::raw("")
            };

            let date_str = mem
                .updated_at
                .as_deref()
                .map(date_short)
                .unwrap_or_default();
            let date_span = Span::styled(
                format!("  {}", date_str),
                Style::default().fg(COLOR_SLATE),
            );

            let line = Line::from(vec![
                Span::styled(cursor, item_title_style),
                Span::styled(star, star_style),
                emb_span,
                Span::styled(mem.title.clone(), item_title_style),
                progress_span,
                att_span,
                date_span,
            ]);

            items.push(ListItem::new(line));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(title_text, title_style));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
