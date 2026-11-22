use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Pane, ViewMode};
use crate::checklist::ChecklistItem;
use crate::format::{datetime_compact, similarity_percentage, word_and_line_count};
use crate::ui::{COLOR_AMBER, COLOR_BORDER, COLOR_EMERALD, COLOR_SLATE, COLOR_VIOLET};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Detail;
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

    let block_title = format!(" [3] Memory Detail [{}] ", app.view_mode.label());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(block_title, title_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let detail = match &app.current_memory {
        Some(d) => d,
        None => {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No memory selected",
                Style::default().fg(COLOR_SLATE),
            )));
            f.render_widget(empty, inner);
            return;
        }
    };

    // Split inner area into: Header (3 lines), Divider (1 line), Main body (Min(0)), Nearest Neighbors (Length(6))
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(6),
        ])
        .split(inner);

    // Metadata lines
    let mut meta_lines = Vec::new();
    let star = if detail.pinned { " ★ Pinned" } else { "" };

    meta_lines.push(Line::from(vec![
        Span::styled(
            &detail.title,
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            star,
            Style::default()
                .fg(COLOR_AMBER)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let (words, lines_count) = word_and_line_count(&detail.body);

    let updated = detail
        .updated_at
        .as_deref()
        .map(datetime_compact)
        .unwrap_or_else(|| "unknown".into());

    let att_text = if detail.attachments_count > 0 {
        format!(" · 📎 {} files ('A' to view)", detail.attachments_count)
    } else {
        " · 📎 0 files ('A' to attach)".to_string()
    };

    let emb_text = if detail.embedded {
        let model = detail.embedding_model.as_deref().unwrap_or("qwen3-embedding:0.6b");
        format!("Vector: Embedded ({})", model)
    } else {
        "Vector: Not embedded ('V' to vectorize)".to_string()
    };
    let emb_color = if detail.embedded {
        COLOR_EMERALD
    } else {
        COLOR_AMBER
    };

    meta_lines.push(Line::from(vec![
        Span::styled(emb_text, Style::default().fg(emb_color)),
        Span::styled(
            format!(" · {}w / {}L", words, lines_count),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!(" · Updated: {}", updated),
            Style::default().fg(COLOR_SLATE),
        ),
        Span::styled(att_text, Style::default().fg(Color::Cyan)),
    ]));

    f.render_widget(
        Paragraph::new(meta_lines).wrap(Wrap { trim: false }),
        chunks[0],
    );

    // Divider
    let divider = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(COLOR_BORDER),
    )));
    f.render_widget(divider, chunks[1]);

    // Body rendering according to ViewMode
    let rendered_lines = match app.view_mode {
        ViewMode::Preview => render_markdown_preview(
            &detail.body,
            &app.checklist_items,
            app.detail_selected_checklist,
            is_focused,
            chunks[2].width as usize,
        ),
        ViewMode::Raw => render_markdown_raw(&detail.body),
    };

    let scroll_y = app.detail_scroll as u16;
    let body_paragraph = Paragraph::new(rendered_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));
    f.render_widget(body_paragraph, chunks[2]);

    // Bottom Pinned Nearest Neighbors Panel
    render_neighbors_panel(f, app, chunks[3]);
}

fn render_neighbors_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(COLOR_BORDER))
        .title(Span::styled(
            " Nearest Semantic Neighbors (Cosine pgvector) ",
            Style::default()
                .fg(COLOR_VIOLET)
                .add_modifier(Modifier::BOLD),
        ));

    let mut lines = Vec::new();
    if app.current_neighbors.is_empty() {
        let is_embedded = app
            .current_memory
            .as_ref()
            .map(|d| d.embedded)
            .unwrap_or(false);
        let msg = if is_embedded {
            " (no nearest neighbors surfaced yet)"
        } else {
            " (memory not yet embedded — press 'V' to compute 1024-dim embedding)"
        };
        lines.push(Line::from(Span::styled(
            msg,
            Style::default().fg(COLOR_SLATE),
        )));
    } else {
        for n in app.current_neighbors.iter().take(4) {
            let score_str = similarity_percentage(n.score);
            let line = Line::from(vec![
                Span::styled(" ▶ ", Style::default().fg(COLOR_EMERALD)),
                Span::styled(
                    format!("{:<15}", score_str),
                    Style::default()
                        .fg(COLOR_EMERALD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" · {} ", n.memory.title),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("(id: {})", n.memory.id),
                    Style::default().fg(COLOR_SLATE),
                ),
            ]);
            lines.push(line);
        }
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn render_markdown_raw(text: &str) -> Vec<Line<'_>> {
    if text.trim().is_empty() {
        return vec![Line::from(Span::styled(
            "(empty memory body — press 'a' to append content, 'E' to edit in $EDITOR)",
            Style::default().fg(COLOR_SLATE),
        ))];
    }

    text.lines()
        .enumerate()
        .map(|(idx, line)| {
            Line::from(vec![
                Span::styled(
                    format!("{:4} │ ", idx + 1),
                    Style::default().fg(COLOR_BORDER),
                ),
                Span::styled(line, Style::default().fg(Color::White)),
            ])
        })
        .collect()
}

fn render_markdown_preview<'a>(
    text: &'a str,
    checklist_items: &[ChecklistItem],
    selected_checklist: Option<usize>,
    pane_focused: bool,
    width: usize,
) -> Vec<Line<'a>> {
    if text.trim().is_empty() {
        return vec![Line::from(Span::styled(
            "(empty memory body — press 'a' to append content, 'E' to edit in $EDITOR)",
            Style::default().fg(COLOR_SLATE),
        ))];
    }

    let mut out = Vec::new();
    let mut in_code_block = false;

    for (line_idx, raw_line) in text.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = raw_line.trim();

        // Fenced code blocks
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default().fg(COLOR_BORDER),
            )));
            continue;
        }

        if in_code_block {
            out.push(Line::from(Span::styled(
                format!("  {}", raw_line),
                Style::default().fg(COLOR_EMERALD),
            )));
            continue;
        }

        // Checklist item matching
        if let Some(item) = checklist_items.iter().find(|i| i.line_number == line_num) {
            let is_selected_item = selected_checklist == Some(item.index);
            let bullet_cursor = if is_selected_item && pane_focused {
                "▶ "
            } else {
                "  "
            };

            let mark_span = if item.completed {
                Span::styled(
                    "[x] ",
                    Style::default()
                        .fg(COLOR_EMERALD)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "[ ] ",
                    Style::default()
                        .fg(COLOR_AMBER)
                        .add_modifier(Modifier::BOLD),
                )
            };

            let text_style = if is_selected_item && pane_focused {
                Style::default()
                    .fg(Color::White)
                    .bg(COLOR_BORDER)
                    .add_modifier(Modifier::BOLD)
            } else if item.completed {
                Style::default()
                    .fg(COLOR_SLATE)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::White)
            };

            let cursor_style = if is_selected_item && pane_focused {
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_BORDER)
            };

            let line = Line::from(vec![
                Span::styled(bullet_cursor, cursor_style),
                Span::styled(item.indent.clone(), Style::default()),
                mark_span,
                Span::styled(item.text.clone(), text_style),
            ]);

            out.push(line);
            continue;
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            out.push(Line::from(Span::styled(
                "─".repeat(width.saturating_sub(2).max(10)),
                Style::default().fg(COLOR_BORDER),
            )));
            continue;
        }

        // Headings
        if trimmed.starts_with("# ") {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
        } else if trimmed.starts_with("## ") {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(COLOR_VIOLET)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if trimmed.starts_with("### ") || trimmed.starts_with("#### ") {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if trimmed.starts_with('>') {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(COLOR_SLATE)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default().fg(Color::LightCyan),
            )));
        } else if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* "))
        {
            let indent = &raw_line[..(raw_line.len() - trimmed.len())];
            out.push(Line::from(vec![
                Span::raw(indent),
                Span::styled("• ", Style::default().fg(COLOR_EMERALD)),
                Span::styled(rest, Style::default().fg(Color::White)),
            ]));
        } else {
            out.push(Line::from(Span::styled(
                raw_line,
                Style::default().fg(Color::White),
            )));
        }
    }

    out
}
