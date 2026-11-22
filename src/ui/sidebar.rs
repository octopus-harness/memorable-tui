use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, Pane};
use crate::models::Scope;
use crate::ui::{COLOR_BORDER, COLOR_SLATE, COLOR_VIOLET};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Sidebar;
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

    let mut items = Vec::new();

    let scopes = Scope::all();
    for (i, scope) in scopes.iter().enumerate() {
        let is_selected = app.sidebar_selected == i;
        let prefix = if is_selected { "▶ " } else { "  " };

        let style = if is_selected {
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let count_str = match scope {
            Scope::All => format!("({})", app.stats.total_memories),
            Scope::Pinned => format!("({})", app.stats.pinned_memories),
            _ => String::new(),
        };

        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{:<15}", scope.label()), style),
            Span::styled(count_str, Style::default().fg(COLOR_SLATE)),
        ]);
        items.push(ListItem::new(line));
    }

    // Divider
    items.push(ListItem::new(Line::from(Span::styled(
        " ───────────────────────",
        Style::default().fg(COLOR_BORDER),
    ))));

    // Pinned Header
    items.push(ListItem::new(Line::from(Span::styled(
        "  PINNED MEMORIES",
        Style::default()
            .fg(COLOR_SLATE)
            .add_modifier(Modifier::BOLD),
    ))));

    if app.pinned_memories.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "   (no pinned memories)",
            Style::default().fg(COLOR_SLATE),
        ))));
    } else {
        for (idx, mem) in app.pinned_memories.iter().enumerate() {
            let actual_idx = scopes.len() + idx;
            let is_selected = app.sidebar_selected == actual_idx;
            let prefix = if is_selected { "▶ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let char_count = mem.title.chars().count();
            let name_truncated = if char_count > 22 {
                let s: String = mem.title.chars().take(21).collect();
                format!("{}…", s)
            } else {
                mem.title.clone()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(name_truncated, style),
            ]);
            items.push(ListItem::new(line));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" [1] Scopes & Pinned ", title_style));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
