use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use tigrs_core::Settings;
use tigrs_git::{discover_repo, recent_commits, commit_diff_text, oid_from_str, CommitInfo};

#[derive(Debug, Parser)]
#[command(name = "tig-rs", version, about = "Experimental Rust rewrite scaffold for Tig")]
pub struct Args {
    /// Number of commits to show
    #[arg(short = 'n', long, default_value_t = 50)]
    limit: usize,
    /// Start path for repository discovery
    #[arg()] 
    path: Option<String>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    let settings = Settings::load().unwrap_or_default();

    let repo = discover_repo(args.path.as_deref()).ok();
    let commits = match repo.as_ref().and_then(|r| recent_commits(r, args.limit).ok()) {
        Some(list) => list,
        None => Vec::new(),
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, commits, settings, repo);

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    if let Err(e) = res { eprintln!("Error: {e}"); }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    commits: Vec<CommitInfo>,
    mut settings: Settings,
    repo: Option<git2::Repository>,
) -> Result<()> {
    let mut idx: usize = 0;
    let mut mode = Mode::List;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(f.size());

            match &mode {
                Mode::List => {
                    let footer = Paragraph::new(Span::raw(
                        format!("Enter: open  q: quit  j/k: move  w: wrap={}  {} commits", if settings.wrap_lines { "on" } else { "off" }, commits.len()),
                    ));
                    f.render_widget(footer, chunks[1]);

                    let items: Vec<ListItem> = commits.iter().map(|c| {
                        let line = format!("{} {} — {}", c.id.clone().bold(), c.summary, c.author);
                        ListItem::new(line)
                    }).collect();
                    let list = List::new(items)
                        .block(Block::default().title("tig-rs — commits").borders(Borders::ALL));
                    f.render_stateful_widget(list, chunks[0], &mut ratatui::widgets::ListState::default().with_selected(Some(idx)));
                }
                Mode::Pager { data } => {
                    let footer = Paragraph::new(Span::raw(
                        "q: back  j/k: scroll  g/G: top/bottom  w: wrap  Tab/p/d: switch",
                    ));
                    f.render_widget(footer, chunks[1]);

                    let block = Block::default().title(data.title.as_str()).borders(Borders::ALL);
                    let mut para = Paragraph::new(data.content.as_str()).block(block);
                    if settings.wrap_lines {
                        para = para.wrap(ratatui::widgets::Wrap { trim: false });
                    }
                    para = para.scroll((data.scroll_pager, 0));
                    f.render_widget(para, chunks[0]);
                }
                Mode::Diff { data } => {
                    let footer = Paragraph::new(Span::raw(
                        "q: back  j/k: scroll  g/G: top/bottom  w: wrap  Tab/p/d: switch",
                    ));
                    f.render_widget(footer, chunks[1]);

                    let block = Block::default().title(data.title.as_str()).borders(Borders::ALL);
                    let mut para = Paragraph::new(data.lines.clone()).block(block);
                    if settings.wrap_lines {
                        para = para.wrap(ratatui::widgets::Wrap { trim: false });
                    }
                    para = para.scroll((data.scroll_diff, 0));
                    f.render_widget(para, chunks[0]);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('w') => {
                        settings.wrap_lines = !settings.wrap_lines;
                        let _ = settings.save();
                    }
                    KeyCode::Char('q') => {
                        match &mode {
                            Mode::List => break,
                            Mode::Pager { .. } | Mode::Diff { .. } => mode = Mode::List,
                        }
                    }
                    KeyCode::Enter => {
                        if let Mode::List = mode {
                            if let Some(repo) = repo.as_ref() {
                                if let Some(commit) = commits.get(idx) {
                                    if let Ok(oid) = oid_from_str(repo, &commit.full_id) {
                                        if let Ok(text) = commit_diff_text(repo, oid) {
                                            let title = format!("{} {}", commit.id, commit.summary);
                                            let data = ViewData {
                                                title,
                                                content: text.clone(),
                                                lines: colorize_diff(&text),
                                                scroll_pager: 0,
                                                scroll_diff: 0,
                                            };
                                            mode = Mode::Pager { data: Box::new(data) };
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Tab => {
                        toggle_view(&mut mode);
                    }
                    KeyCode::Char('p') => {
                        to_pager(&mut mode);
                    }
                    KeyCode::Char('d') => {
                        to_diff(&mut mode);
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        match &mut mode {
                            Mode::List => {
                                idx = idx.saturating_add(1).min(commits.len().saturating_sub(1));
                            }
                            Mode::Pager { data } => {
                                data.scroll_pager = data.scroll_pager.saturating_add(1);
                            }
                            Mode::Diff { data } => {
                                data.scroll_diff = data.scroll_diff.saturating_add(1);
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match &mut mode {
                            Mode::List => idx = idx.saturating_sub(1),
                            Mode::Pager { data } => data.scroll_pager = data.scroll_pager.saturating_sub(1),
                            Mode::Diff { data } => data.scroll_diff = data.scroll_diff.saturating_sub(1),
                        }
                    }
                    KeyCode::Char('g') => {
                        match &mut mode {
                            Mode::Pager { data } => data.scroll_pager = 0,
                            Mode::Diff { data } => data.scroll_diff = 0,
                            _ => {}
                        }
                    }
                    KeyCode::Char('G') => {
                        match &mut mode {
                            Mode::Pager { data } => data.scroll_pager = u16::MAX,
                            Mode::Diff { data } => data.scroll_diff = u16::MAX,
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}

trait ListStateExt {
    fn with_selected(self, selected: Option<usize>) -> Self;
}

impl ListStateExt for ratatui::widgets::ListState {
    fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.select(selected);
        self
    }
}

enum Mode {
    List,
    Pager { data: Box<ViewData> },
    Diff { data: Box<ViewData> },
}

#[derive(Clone)]
struct ViewData {
    title: String,
    content: String,
    lines: Vec<Line<'static>>,
    scroll_pager: u16,
    scroll_diff: u16,
}

fn colorize_diff(input: &str) -> Vec<Line<'static>> {
    input
        .lines()
        .map(|l| {
            if l.starts_with("+++") || l.starts_with("---") || l.starts_with("diff --git") {
                Line::from(Span::styled(l.to_string(), Style::new().bold()))
            } else if l.starts_with("@@") {
                Line::from(Span::styled(l.to_string(), Style::new().yellow()))
            } else if l.starts_with('+') {
                Line::from(Span::styled(l.to_string(), Style::new().green()))
            } else if l.starts_with('-') {
                Line::from(Span::styled(l.to_string(), Style::new().red()))
            } else {
                Line::from(Span::raw(l.to_string()))
            }
        })
        .collect()
}

fn toggle_view(mode: &mut Mode) {
    match mode {
        Mode::Pager { data } => {
            *mode = Mode::Diff { data: data.clone() };
        }
        Mode::Diff { data } => {
            *mode = Mode::Pager { data: data.clone() };
        }
        _ => {}
    }
}

fn to_pager(mode: &mut Mode) {
    if let Mode::Diff { data } = mode {
        *mode = Mode::Pager { data: data.clone() };
    }
}

fn to_diff(mode: &mut Mode) {
    if let Mode::Pager { data } = mode {
        *mode = Mode::Diff { data: data.clone() };
    }
}

