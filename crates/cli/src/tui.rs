use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use openzax_core::{agent::{Agent, AgentConfig}, event::{Event as OzEvent, EventBus}, storage::Storage};
use uuid::Uuid;
use chrono::Utc;

// ─── Colours ────────────────────────────────────────────────────────────────

const BLUE: Color   = Color::Rgb(100, 180, 255);
const GOLD: Color   = Color::Rgb(255, 180, 60);
const DIM:  Color   = Color::Rgb(70, 70, 90);
const TEXT: Color   = Color::Rgb(220, 220, 235);
const MUTED: Color  = Color::Rgb(120, 120, 150);
const BG:   Color   = Color::Rgb(14, 14, 20);
const BOX:  Color   = Color::Rgb(45, 45, 65);
const _ERR: Color   = Color::Rgb(255, 80, 80);
const _OK:  Color   = Color::Rgb(80, 220, 120);

// ─── Message model ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone)]
pub struct Message {
    pub role:    Role,
    pub content: String,
}

impl Message {
    fn user(content: impl Into<String>)      -> Self { Self { role: Role::User,      content: content.into() } }
    fn assistant(content: impl Into<String>) -> Self { Self { role: Role::Assistant, content: content.into() } }
    fn system(content: impl Into<String>)    -> Self { Self { role: Role::System,    content: content.into() } }
}

// ─── App state ──────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum AppState {
    Idle,
    #[allow(dead_code)]
    Thinking,
    Streaming,
}

pub struct App {
    messages:        Vec<Message>,
    input:           String,
    cursor_pos:      usize,
    state:           AppState,
    scroll:          usize,
    model_name:      String,
    streaming_buf:   Arc<Mutex<String>>,
    done_flag:       Arc<Mutex<bool>>,
}

impl App {
    pub fn new(model_name: &str) -> Self {
        Self {
            messages: vec![
                Message::system("OpenZax started. Type a message and press Enter.".to_string()),
            ],
            input:          String::new(),
            cursor_pos:     0,
            state:          AppState::Idle,
            scroll:         0,
            model_name:     model_name.to_string(),
            streaming_buf:  Arc::new(Mutex::new(String::new())),
            done_flag:      Arc::new(Mutex::new(false)),
        }
    }

    fn insert_char(&mut self, c: char) {
        let char_len = c.len_utf8();
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += char_len;
    }

    fn delete_back(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.remove(prev);
            self.cursor_pos = prev;
        }
    }

    fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            let next = self.input[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.input.len());
            self.cursor_pos = next;
        }
    }

    fn take_input(&mut self) -> String {
        let s = self.input.clone();
        self.input.clear();
        self.cursor_pos = 0;
        s
    }

    fn push_msg(&mut self, msg: Message) {
        self.messages.push(msg);
        self.scroll_to_bottom_hint();
    }

    fn scroll_to_bottom_hint(&mut self) {
        self.scroll = usize::MAX;
    }

    fn scroll_up(&mut self)   { self.scroll = self.scroll.saturating_sub(1); }
    fn scroll_down(&mut self) { self.scroll = self.scroll.saturating_add(1); }

    fn flush_stream(&mut self) {
        let chunk = {
            let mut buf = self.streaming_buf.lock().unwrap();
            let s = buf.clone();
            buf.clear();
            s
        };
        if !chunk.is_empty() {
            if let Some(last) = self.messages.last_mut() {
                if last.role == Role::Assistant {
                    last.content.push_str(&chunk);
                    return;
                }
            }
            self.messages.push(Message::assistant(chunk));
            self.scroll_to_bottom_hint();
        }
    }

    fn check_done(&mut self) -> bool {
        let done = *self.done_flag.lock().unwrap();
        if done {
            *self.done_flag.lock().unwrap() = false;
            true
        } else {
            false
        }
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Background
    f.render_widget(
        Block::default().style(Style::default().bg(BG)),
        area,
    );

    // Layout: header(3) + messages(fill) + status(1) + input(3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_messages(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);
    render_input(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(30), Constraint::Fill(1)])
        .split(area);

    // Left: branding
    let brand = Paragraph::new(Line::from(vec![
        Span::styled("open", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("zax", Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
    ]))
    .style(Style::default().bg(BG))
    .alignment(Alignment::Left)
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(BOX)));

    f.render_widget(brand, cols[0]);

    // Centre: model pill
    let model_label = Paragraph::new(Line::from(vec![
        Span::styled(" ◈ ", Style::default().fg(BLUE)),
        Span::styled(&app.model_name, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
    ]))
    .style(Style::default().bg(BG))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(BOX)));

    f.render_widget(model_label, cols[1]);

    // Right: state indicator
    let (state_icon, state_color) = match app.state {
        AppState::Idle      => ("● idle",      DIM),
        AppState::Thinking  => ("◆ thinking…", GOLD),
        AppState::Streaming => ("▍ streaming", BLUE),
    };

    let right = Paragraph::new(Span::styled(state_icon, Style::default().fg(state_color)))
        .style(Style::default().bg(BG))
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(BOX)));

    f.render_widget(right, cols[2]);
}

fn render_messages(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let inner_w = area.width.saturating_sub(4) as usize;

    // Build rendered lines for every message
    let mut all_lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        match msg.role {
            Role::System => {
                all_lines.push(Line::default());
                all_lines.push(Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(MUTED)),
                    Span::styled(msg.content.as_str(), Style::default().fg(MUTED)),
                ]));
                all_lines.push(Line::default());
            }
            Role::User => {
                all_lines.push(Line::default());
                all_lines.push(Line::from(vec![
                    Span::styled("  ▸ ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
                    Span::styled("You", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
                ]));
                // Word-wrap the content
                for wrapped in wrap_text(&msg.content, inner_w) {
                    all_lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(wrapped, Style::default().fg(TEXT)),
                    ]));
                }
                all_lines.push(Line::default());
            }
            Role::Assistant => {
                all_lines.push(Line::from(vec![
                    Span::styled("  ▪ ", Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
                    Span::styled("OpenZax", Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
                ]));
                for wrapped in wrap_text(&msg.content, inner_w) {
                    all_lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(wrapped, Style::default().fg(TEXT)),
                    ]));
                }
                all_lines.push(Line::default());
            }
        }
    }

    let total = all_lines.len();
    let visible = area.height as usize;

    // Clamp scroll
    let max_scroll = total.saturating_sub(visible);
    if app.scroll >= usize::MAX / 2 {
        app.scroll = max_scroll;
    } else {
        app.scroll = app.scroll.min(max_scroll);
    }

    let text = Text::from(all_lines);

    let para = Paragraph::new(text)
        .style(Style::default().bg(BG).fg(TEXT))
        .scroll((app.scroll as u16, 0));

    f.render_widget(para, area);
}

fn render_status_bar(f: &mut Frame, _app: &App, area: ratatui::layout::Rect) {
    let line = Line::from(vec![
        Span::styled(" ↑↓ scroll  ", Style::default().fg(DIM)),
        Span::styled("Ctrl+C", Style::default().fg(MUTED)),
        Span::styled(" quit  ", Style::default().fg(DIM)),
        Span::styled("Enter", Style::default().fg(MUTED)),
        Span::styled(" send  ", Style::default().fg(DIM)),
        Span::styled("/help", Style::default().fg(GOLD)),
        Span::styled(" commands", Style::default().fg(DIM)),
    ]);

    let bar = Paragraph::new(line)
        .style(Style::default().bg(Color::Rgb(18, 18, 28)).fg(DIM))
        .alignment(Alignment::Left);

    f.render_widget(bar, area);
}

fn render_input(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (border_color, prompt_color) = if app.state == AppState::Idle {
        (BLUE, BLUE)
    } else {
        (GOLD, GOLD)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Prompt + text + cursor
    let before_cursor = &app.input[..app.cursor_pos];
    let cursor_char   = app.input[app.cursor_pos..].chars().next().unwrap_or(' ');
    let after_cursor  = if app.cursor_pos < app.input.len() {
        &app.input[app.cursor_pos + cursor_char.len_utf8()..]
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::styled("❯ ", Style::default().fg(prompt_color).add_modifier(Modifier::BOLD)),
        Span::styled(before_cursor, Style::default().fg(TEXT)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().fg(BG).bg(TEXT),
        ),
        Span::styled(after_cursor, Style::default().fg(TEXT)),
    ]);

    let para = Paragraph::new(line)
        .style(Style::default().bg(BG))
        .wrap(Wrap { trim: false });

    f.render_widget(para, inner);
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut cur = String::new();
        let mut cur_len = 0usize;
        for word in raw_line.split_whitespace() {
            let wlen = word.chars().count();
            if cur_len == 0 {
                cur.push_str(word);
                cur_len = wlen;
            } else if cur_len + 1 + wlen <= width {
                cur.push(' ');
                cur.push_str(word);
                cur_len += 1 + wlen;
            } else {
                lines.push(cur.clone());
                cur = word.to_string();
                cur_len = wlen;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
    }
    lines
}

// ─── Built-in commands ──────────────────────────────────────────────────────

fn handle_command(app: &mut App, cmd: &str) -> bool {
    let cmd = cmd.trim();
    match cmd {
        "/help" | "/h" => {
            app.push_msg(Message::system(
                "/help  show commands  |  /clear  clear chat  |  /model  show model  |  /exit  quit  |  ↑↓ scroll"
                    .to_string(),
            ));
            true
        }
        "/clear" => {
            app.messages.clear();
            app.push_msg(Message::system("Chat cleared.".to_string()));
            true
        }
        "/model" => {
            let m = app.model_name.clone();
            app.push_msg(Message::system(format!("Current model: {}", m)));
            true
        }
        "/exit" | "/quit" => {
            // signal exit via special system message
            app.push_msg(Message { role: Role::System, content: "__EXIT__".to_string() });
            true
        }
        _ => false,
    }
}

// ─── Main TUI entry point ───────────────────────────────────────────────────

pub async fn run_tui(
    model_name: String,
    api_key: Option<String>,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, model_name, api_key, db_path).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model_name: String,
    api_key: Option<String>,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut app = App::new(&model_name);

    // ── Agent setup (optional – if no api_key, run in demo mode) ────────────
    let (agent_opt, event_bus_opt, storage_opt, conv_id): (
        Option<Arc<Agent>>,
        Option<EventBus>,
        Option<Storage>,
        Uuid,
    ) = if let Some(key) = api_key {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let event_bus = EventBus::default();
        let config = AgentConfig {
            api_key: Some(key),
            model: model_name.clone(),
            ..Default::default()
        };
        let agent = Arc::new(Agent::new(config, event_bus.clone()));
        let storage = Storage::new(&db_path)?;
        let conv_id = Uuid::new_v4();
        storage.create_conversation(conv_id)?;
        (Some(agent), Some(event_bus), Some(storage), conv_id)
    } else {
        app.push_msg(Message::system(
            "No API key set. Running in demo mode. Set OPENZAX_API_KEY or use --api-key.".to_string(),
        ));
        (None, None, None, Uuid::new_v4())
    };

    // ── Background: drain event bus into streaming_buf ───────────────────────
    if let Some(ref eb) = event_bus_opt {
        let mut rx   = eb.subscribe();
        let buf_ref  = Arc::clone(&app.streaming_buf);
        let done_ref = Arc::clone(&app.done_flag);

        tokio::spawn(async move {
            while let Ok(ev) = rx.recv().await {
                match ev {
                    OzEvent::AgentTokenStream { token, .. } => {
                        buf_ref.lock().unwrap().push_str(&token);
                    }
                    OzEvent::AgentOutput { .. } => {
                        *done_ref.lock().unwrap() = true;
                    }
                    _ => {}
                }
            }
        });
    }

    // ── Main event loop ──────────────────────────────────────────────────────
    loop {
        // Flush any streamed tokens into the message list
        if app.state == AppState::Streaming {
            app.flush_stream();
            if app.check_done() {
                app.state = AppState::Idle;
            }
        }

        terminal.draw(|f| render(f, &mut app))?;

        // Poll with short timeout so streaming updates render smoothly
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, not release
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                
                // Global: Ctrl+C to quit
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }

                match app.state {
                    AppState::Idle => match key.code {
                        KeyCode::Enter => {
                            let text = app.take_input();
                            if text.trim().is_empty() {
                                continue;
                            }

                            // Check built-in commands
                            if text.trim().starts_with('/') {
                                handle_command(&mut app, text.trim());
                                // Check for exit signal
                                if app.messages.last().map(|m| m.content == "__EXIT__").unwrap_or(false) {
                                    break;
                                }
                                continue;
                            }

                            app.push_msg(Message::user(&text));

                            if let (Some(ref agent), Some(ref storage)) = (&agent_opt, &storage_opt) {
                                // Save to DB
                                storage.save_message(Uuid::new_v4(), conv_id, "user", &text).ok();

                                // Publish event
                                if let Some(ref eb) = event_bus_opt {
                                    eb.publish(OzEvent::UserInput {
                                        session_id:  conv_id,
                                        content:     text.clone(),
                                        attachments: vec![],
                                        timestamp:   Utc::now(),
                                    }).ok();
                                }

                                // Push placeholder for streaming
                                app.messages.push(Message::assistant(String::new()));
                                app.scroll_to_bottom_hint();
                                app.state = AppState::Streaming;

                                // Spawn agent call
                                let agent_arc = Arc::clone(agent);
                                tokio::spawn(async move {
                                    agent_arc.process_streaming(&text).await.ok();
                                });
                            } else {
                                // Demo mode echo
                                app.push_msg(Message::assistant(format!(
                                    "Demo mode — I received: \"{}\".\nSet OPENZAX_API_KEY to enable the AI.",
                                    text
                                )));
                            }
                        }

                        KeyCode::Char(c) => app.insert_char(c),
                        KeyCode::Backspace => app.delete_back(),
                        KeyCode::Left  => app.cursor_left(),
                        KeyCode::Right => app.cursor_right(),
                        KeyCode::Up    => app.scroll_up(),
                        KeyCode::Down  => app.scroll_down(),

                        KeyCode::Home => app.cursor_pos = 0,
                        KeyCode::End  => app.cursor_pos = app.input.len(),

                        _ => {}
                    },

                    // While thinking/streaming: allow scroll only
                    _ => match key.code {
                        KeyCode::Up   => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        _ => {}
                    },
                }
            }
        }
    }

    Ok(())
}
