use arboard::Clipboard;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{TableState},
};

use std::{
    fs,
    path::Path,
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use crate::data::{Exploit, SearchSploitOutput};

#[derive(PartialEq)]
pub enum InputMode {
    Insert,
    Normal,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    Results,
    Preview,
}

pub enum WorkerMessage {
    SearchResults(Result<Vec<Exploit>, String>),
}

pub struct App {
    pub query: String,
    pub input_mode: InputMode,
    pub focus: Focus,
    pub results: Vec<Exploit>,
    pub table_state: TableState,
    pub preview_text: Text<'static>,
    pub preview_scroll: u16,
    pub preview_max_scroll: u16,
    pub status_msg: String,
    pub is_searching: bool,
    pub search_tx: Sender<String>,
    pub worker_rx: Receiver<WorkerMessage>,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
}

impl App {
    pub fn new() -> Self {
        let (search_tx, search_rx) = mpsc::channel::<String>();
        let (worker_tx, worker_rx) = mpsc::channel::<WorkerMessage>();

        thread::spawn(move || {
            while let Ok(query) = search_rx.recv() {
                let output = Command::new("searchsploit")
                    .args(["-j", &query])
                    .output();

                let msg = match output {
                    Ok(out) => match serde_json::from_slice::<SearchSploitOutput>(&out.stdout) {
                        Ok(parsed) => WorkerMessage::SearchResults(Ok(parsed.results)),
                        Err(e) => WorkerMessage::SearchResults(Err(format!("JSON parse error: {e}"))),
                    },
                    Err(e) => WorkerMessage::SearchResults(Err(format!("Failed to run searchsploit: {e}"))),
                };

                if worker_tx.send(msg).is_err() {
                    break;
                }
            }
	});

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self {
            query: String::new(),
            input_mode: InputMode::Insert,
            focus: Focus::Results,
            results: Vec::new(),
            table_state: TableState::default(),
            preview_text: Text::from("Type a search query and press Enter."),
            preview_scroll: 0,
            preview_max_scroll: 0,
            status_msg: String::from("Ready"),
            is_searching: false,
            search_tx,
            worker_rx,
            syntax_set,
            theme,
        }
    }

    pub fn trigger_search(&mut self) {
        if self.query.trim().is_empty() {
            return;
        }
        self.is_searching = true;
        self.status_msg = format!("Searching for '{}'...", self.query);
        let _ = self.search_tx.send(self.query.clone());
        self.input_mode = InputMode::Normal;
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Results => Focus::Preview,
            Focus::Preview => Focus::Results,
        };
    }

    pub fn scroll_preview(&mut self, delta: i32) {
        self.preview_scroll = ((self.preview_scroll as i32 + delta)
            .clamp(0, self.preview_max_scroll as i32)) as u16;
    }

    pub fn poll_worker(&mut self) {
        while let Ok(msg) = self.worker_rx.try_recv() {
            self.is_searching = false;
            match msg {
                WorkerMessage::SearchResults(Ok(exploits)) => {
                    self.status_msg = format!("Found {} exploits", exploits.len());
                    self.results = exploits;
                    if !self.results.is_empty() {
                        self.table_state.select(Some(0));
                        self.load_preview();
                    } else {
                        self.table_state.select(None);
                        self.preview_text = Text::from("No results matched your query.");
                    }
                }
                WorkerMessage::SearchResults(Err(err)) => {
                    self.status_msg = err;
                }
            }
        }
    }

    pub fn highlight_code(&self, code: &str, file_path: &str) -> Text<'static> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");

        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines: Vec<Line<'static>> = Vec::new();

        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    let spans: Vec<Span<'static>> = ranges
                        .into_iter()
                        .map(|(style, text)| {
                            let fg = Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            );
                            Span::styled(
                                text.trim_end_matches(['\r', '\n']).to_string(),
                                Style::default().fg(fg),
                            )
                        })
                        .collect();
                    lines.push(Line::from(spans));
                }
                Err(_) => {
                    lines.push(Line::from(line.trim_end_matches(['\r', '\n']).to_string()));
                }
            }
	}

        Text::from(lines)
    }

    pub fn load_preview(&mut self) {
        self.preview_scroll = 0;
        if let Some(idx) = self.table_state.selected() {
            if let Some(exploit) = self.results.get(idx) {
                let path = exploit.path.clone();
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        self.preview_text = self.highlight_code(&content, &path);
                    }
                    Err(e) => {
                        self.preview_text = Text::from(format!("Error reading {path}: {e}"));
                    }
                }
            }
	}
    }

    pub fn selected_exploit(&self) -> Option<&Exploit> {
        self.table_state.selected().and_then(|i| self.results.get(i))
    }

    pub fn copy_path_to_clipboard(&mut self) {
        if let Some(exploit) = self.selected_exploit() {
            let path = exploit.path.clone();
            match Clipboard::new().and_then(|mut cb| cb.set_text(&path)) {
                Ok(_) => self.status_msg = format!("Copied path: {path}"),
                Err(e) => self.status_msg = format!("Clipboard error: {e}"),
            }
	}
    }

    pub fn mirror_exploit(&mut self) {
        if let Some(exploit) = self.selected_exploit() {
            let edb_id = exploit.edb_id.clone();
            match Command::new("searchsploit").args(["-m", &edb_id]).output() {
                Ok(out) if out.status.success() => {
                    self.status_msg = format!("Mirrored EDB-{edb_id} to current directory");
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    self.status_msg = format!("Mirror failed: {}", err.trim());
                }
                Err(e) => self.status_msg = format!("Failed to execute mirror: {e}"),
            }
	}
    }

    pub fn next_row(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % self.results.len(),
            None => 0,
        };
        self.table_state.select(Some(i));
        self.load_preview();
    }

    pub fn prev_row(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + self.results.len() - 1) % self.results.len(),
            None => 0,
        };
        self.table_state.select(Some(i));
        self.load_preview();
    }
}
