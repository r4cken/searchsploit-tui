// Copyright (C) 2026 Ellie Lilja
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, Focus, InputMode};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    let main_panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(vertical[1]);

    // 1. Search Bar
    let (search_title, border_color) = match app.input_mode {
        InputMode::Insert => (
            " Search [INSERT] (Enter: Run search, Esc: normal mode) ",
            Color::Cyan,
        ),
        InputMode::Normal => (
            " Search [NORMAL] (Press '/' or 'i' to edit) ",
            Color::DarkGray,
        ),
    };

    let search_bar = Paragraph::new(app.query.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(search_title),
    );

    frame.render_widget(search_bar, vertical[0]);

    // Focused pane gets a highlighted border, same as the search bar for normal/insert mode changes
    let (results_border, preview_border) = match app.focus {
        Focus::Results => (Color::Cyan, Color::DarkGray),
        Focus::Preview => (Color::DarkGray, Color::Cyan),
    };

    // 2. Results Table
    let rows: Vec<Row> = app
        .results
        .iter()
        .map(|e| {
            Row::new(vec![
                e.edb_id.clone(),
                e.platform.clone(),
                e.exploit_type.clone(),
                e.title.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(15),
        ],
    )
    .header(
        Row::new(vec!["EDB-ID", "Platform", "Type", "Title"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        ),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(results_border))
            .title(" Exploits "),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(table, main_panes[0], &mut app.table_state);

    // 3. Syntax-Highlighted Preview Pane
    let preview_title = app
        .selected_exploit()
        .map(|e| format!(" Preview: {} ", e.path))
        .unwrap_or_else(|| " Preview ".to_string());

    let preview = Paragraph::new(app.preview_text.clone())
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(preview_border))
                .title(preview_title),
        );
    // Wrap-aware scroll bound: line_count adds the block's borders, which cancel
    // against the full pane height.
    app.preview_max_scroll = preview
        .line_count(main_panes[1].width.saturating_sub(2))
        .saturating_sub(main_panes[1].height as usize) as u16;

    frame.render_widget(preview, main_panes[1]);

    // 4. Bottom Status Bar
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status_msg),
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
        Span::raw(
            "  Tab or ←/→ or h/l: Switch Pane Focus | ↑/↓ or j/k: Nav/Scroll | y: Copy Path | m: Mirror | q: Quit",
        ),
    ]);

    frame.render_widget(Paragraph::new(status_line), vertical[2]);
}
