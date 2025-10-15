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
                    let mut state = list_state(Some(idx));
                    f.render_stateful_widget(list, chunks[0], &mut state);
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

fn list_state(selected: Option<usize>) -> ratatui::widgets::ListState {
    let mut s = ratatui::widgets::ListState::default();
    s.select(selected);
    s
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
    let mut lang: Option<String> = None;
    let mut out = Vec::new();
    for l in input.lines() {
        if l.starts_with("diff --git ") {
            out.push(Line::from(Span::styled(l.to_string(), Style::new().bold())));
            continue;
        }
        if l.starts_with("+++") || l.starts_with("---") {
            // Try to infer language from file path (b/<path> or a/<path>)
            if let Some(path) = l.split_whitespace().nth(1) {
                // strip a/ or b/
                let p = path.trim_start_matches("a/").trim_start_matches("b/");
                if let Some(ext) = p.rsplit('.').next() {
                    lang = Some(ext.to_string());
                }
            }
            out.push(Line::from(Span::styled(l.to_string(), Style::new().bold())));
            continue;
        }
        if l.starts_with("@@") {
            out.push(Line::from(Span::styled(l.to_string(), Style::new().yellow())));
            continue;
        }

        // Content lines
        if let Some(rest) = l.strip_prefix('+') {
            let mut spans = Vec::new();
            spans.push(Span::styled("+".to_string(), Style::new().green()));
            spans.extend(highlight_code(rest, lang.as_deref()));
            out.push(Line::from(spans));
            continue;
        }
        if let Some(rest) = l.strip_prefix('-') {
            let mut spans = Vec::new();
            spans.push(Span::styled("-".to_string(), Style::new().red()));
            spans.extend(highlight_code(rest, lang.as_deref()));
            out.push(Line::from(spans));
            continue;
        }
        if let Some(rest) = l.strip_prefix(' ') {
            let mut spans = Vec::new();
            spans.push(Span::raw(" ".to_string()));
            spans.extend(highlight_code(rest, lang.as_deref()));
            out.push(Line::from(spans));
            continue;
        }

        // Fallback raw line
        out.push(Line::from(Span::raw(l.to_string())));
    }
    out
}

fn highlight_code(line: &str, ext: Option<&str>) -> Vec<Span<'static>> {
    match ext.unwrap_or("") {
        "rs" => highlight_with_rules(line, Lang::Rust),
        "c" | "h" | "hpp" | "hh" | "cpp" | "cc" | "cxx" => highlight_with_rules(line, Lang::Cfamily),
        "py" => highlight_with_rules(line, Lang::Python),
        "js" | "jsx" | "ts" | "tsx" => highlight_with_rules(line, Lang::JsTs),
        "go" => highlight_with_rules(line, Lang::Go),
        "sh" | "bash" | "zsh" => highlight_with_rules(line, Lang::Shell),
        _ => vec![Span::raw(line.to_string())],
    }
}

#[derive(Copy, Clone)]
enum Lang { Rust, Cfamily, Python, JsTs, Go, Shell }

fn highlight_with_rules(line: &str, lang: Lang) -> Vec<Span<'static>> {
    // Simple, single-line highlighter: strings, comments, keywords, numbers.
    // Comments (//, #) take precedence over keyword/number highlighting.
    // Strings are highlighted as a whole; no escapes handling.
    let (comment_start, comment_marker) = match lang {
        Lang::Python | Lang::Shell => (line.find('#'), '#'),
        _ => (line.find("//"), '/'),
    };

    let (code_part, comment_part) = match comment_start {
        Some(idx) => (&line[..idx], Some(&line[idx..])),
        None => (line, None),
    };

    let mut spans = Vec::new();
    spans.extend(highlight_code_tokens(code_part, lang));
    if let Some(comment) = comment_part {
        // Color comments faintly using blue to stand out
        spans.push(Span::styled(comment.to_string(), Style::new().blue()));
    }
    spans
}

fn is_ident_char(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

fn highlight_code_tokens(s: &str, lang: Lang) -> Vec<Span<'static>> {
    let keywords: &'static [&'static str] = match lang {
        Lang::Rust => &[
            "as","break","const","continue","crate","else","enum","extern","false","fn","for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return","self","Self","static","struct","super","trait","true","type","unsafe","use","where","while","async","await","dyn",
        ],
        Lang::Cfamily => &[
            "auto","break","case","char","const","continue","default","do","double","else","enum","extern","float","for","goto","if","inline","int","long","register","restrict","return","short","signed","sizeof","static","struct","switch","typedef","union","unsigned","void","volatile","while","namespace","class","template","typename","using","public","private","protected","virtual","override","constexpr","nullptr","true","false",
        ],
        Lang::Python => &[
            "False","None","True","and","as","assert","async","await","break","class","continue","def","del","elif","else","except","finally","for","from","global","if","import","in","is","lambda","nonlocal","not","or","pass","raise","return","try","while","with","yield",
        ],
        Lang::JsTs => &[
            "break","case","catch","class","const","continue","debugger","default","delete","do","else","export","extends","finally","for","function","if","import","in","instanceof","let","new","return","super","switch","this","throw","try","typeof","var","void","while","with","yield","await","async","enum","interface","type","implements","true","false",
        ],
        Lang::Go => &[
            "break","case","chan","const","continue","default","defer","else","fallthrough","for","func","go","goto","if","import","interface","map","package","range","return","select","struct","switch","type","var","true","false","iota",
        ],
        Lang::Shell => &[
            "if","then","else","elif","fi","for","in","do","done","case","esac","while","until","function","select","time","coproc","true","false",
        ],
    };

    let mut spans = Vec::new();
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        let c = s[i..].chars().next().unwrap();
        // Strings
        if c == '"' || (c == '\'' && !matches!(lang, Lang::Rust | Lang::Cfamily)) {
            let quote = c;
            let mut j = i + c.len_utf8();
            while j < bytes.len() {
                let ch = s[j..].chars().next().unwrap();
                let prev = if j > 0 { s[..j].chars().last().unwrap_or('\0') } else { '\0' };
                let end = ch == quote && prev != '\\';
                j += ch.len_utf8();
                if end { break; }
            }
            spans.push(Span::styled(s[i..j].to_string(), Style::new().yellow()));
            i = j;
            continue;
        }
        // Numbers
        if c.is_ascii_digit() {
            let mut j = i + c.len_utf8();
            while j < bytes.len() {
                let ch = s[j..].chars().next().unwrap();
                if ch.is_ascii_digit() || ch == '.' { j += ch.len_utf8(); } else { break; }
            }
            spans.push(Span::styled(s[i..j].to_string(), Style::new().cyan()));
            i = j;
            continue;
        }
        // Identifiers and keywords
        if is_ident_char(c) {
            let mut j = i + c.len_utf8();
            while j < bytes.len() {
                let ch = s[j..].chars().next().unwrap();
                if is_ident_char(ch) { j += ch.len_utf8(); } else { break; }
            }
            let tok = &s[i..j];
            if keywords.contains(&tok) {
                spans.push(Span::styled(tok.to_string(), Style::new().magenta()));
            } else {
                spans.push(Span::raw(tok.to_string()));
            }
            i = j;
            continue;
        }
        // Whitespace or punct
        spans.push(Span::raw(c.to_string()));
        i += c.len_utf8();
    }
    spans
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
