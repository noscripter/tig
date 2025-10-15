use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use tigrs_core::Settings;
use tigrs_git::{discover_repo, recent_commits, commit_diff_text, oid_from_str, CommitInfo};
use tigrs_tui::{Router, Transition, View, TuiFrame};

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
    settings: Settings,
    repo: Option<git2::Repository>,
) -> Result<()> {
    let mut state = AppState { settings, repo, commits };
    let root: Box<dyn View<AppState>> = Box::new(ListView { idx: 0 });
    let mut router = Router::new(root);

    loop {
        terminal.draw(|f| {
            let area = f.size();
            router.render(f, area, &state);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            let ev = event::read()?;
            if router.handle_event(&ev, &mut state) { break; }
        }
    }
    Ok(())
}

fn list_state(selected: Option<usize>) -> ratatui::widgets::ListState {
    let mut s = ratatui::widgets::ListState::default();
    s.select(selected);
    s
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

fn colorize_diff_basic(input: &str) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for l in input.lines() {
        if l.starts_with("diff --git ") || l.starts_with("+++") || l.starts_with("---") {
            out.push(Line::from(Span::styled(l.to_string(), Style::new().bold())));
            continue;
        }
        if l.starts_with("@@") {
            out.push(Line::from(Span::styled(l.to_string(), Style::new().yellow())));
            continue;
        }
        if let Some(rest) = l.strip_prefix('+') {
            out.push(Line::from(vec![
                Span::styled("+".to_string(), Style::new().green()),
                Span::raw(rest.to_string()),
            ]));
            continue;
        }
        if let Some(rest) = l.strip_prefix('-') {
            out.push(Line::from(vec![
                Span::styled("-".to_string(), Style::new().red()),
                Span::raw(rest.to_string()),
            ]));
            continue;
        }
        if let Some(rest) = l.strip_prefix(' ') {
            out.push(Line::from(vec![
                Span::raw(" ".to_string()),
                Span::raw(rest.to_string()),
            ]));
            continue;
        }
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

// ----------------- App State and Views (router-based) -----------------

struct AppState {
    settings: Settings,
    repo: Option<git2::Repository>,
    commits: Vec<CommitInfo>,
}

struct ListView { idx: usize }
impl View<AppState> for ListView {
    fn title(&self) -> String { "tig-rs — commits".into() }
    fn render(&mut self, f: &mut TuiFrame<'_>, area: Rect, state: &AppState) {
        // Layout: content + footer (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        // Colored footer
        let mut fs = Vec::new();
        fs.push(Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": open  "));
        fs.push(Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": quit  "));
        fs.push(Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": move  "));
        fs.push(Span::styled("w", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(format!(": wrap={}  ", if state.settings.wrap_lines { "on" } else { "off" })));
        fs.push(Span::styled("y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(format!(": syn={}  ", if state.settings.syntax_highlight { "on" } else { "off" })));
        fs.push(Span::raw(format!("{} commits", state.commits.len())));
        f.render_widget(Paragraph::new(Line::from(fs)), chunks[1]);

        let items: Vec<ListItem> = state.commits.iter().map(|c| {
            let mut spans: Vec<Span> = Vec::new();
            // ID highlighted
            let id_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
            spans.push(Span::styled(c.id.clone(), id_style));
            spans.push(Span::raw(" "));

            // Summary with simple keyword-based coloring
            let sum_lower = c.summary.to_lowercase();
            let mut sum_style = Style::default();
            if sum_lower.starts_with("feat") {
                sum_style = Style::default().fg(Color::Green);
            } else if sum_lower.starts_with("fix") {
                sum_style = Style::default().fg(Color::Red);
            } else if sum_lower.starts_with("docs") {
                sum_style = Style::default().fg(Color::Blue);
            } else if sum_lower.starts_with("refactor") {
                sum_style = Style::default().fg(Color::Magenta);
            }
            spans.push(Span::styled(c.summary.clone(), sum_style));

            // Author dimmed
            spans.push(Span::raw(" — "));
            spans.push(Span::styled(c.author.clone(), Style::default().fg(Color::DarkGray)));

            ListItem::new(Line::from(spans))
        }).collect();
        let list = List::new(items)
            .block(Block::default().title(self.title()).borders(Borders::ALL))
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        let selected = if state.commits.is_empty() { None } else { Some(self.idx) };
        let mut selection = list_state(selected);
        f.render_stateful_widget(list, chunks[0], &mut selection);
    }
    fn on_event(&mut self, ev: &Event, state: &mut AppState) -> Transition<AppState> {
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Char('w') => {
                    state.settings.wrap_lines = !state.settings.wrap_lines;
                    let _ = state.settings.save();
                }
                KeyCode::Char('q') => return Transition::Quit,
                KeyCode::Enter => {
                    if let (Some(repo), Some(commit)) = (state.repo.as_ref(), state.commits.get(self.idx)) {
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
                                // Open Diff view by default so highlighting is visible immediately
                                return Transition::Push(Box::new(DiffView { data }));
                            }
                        }
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.idx = self.idx.saturating_add(1).min(state.commits.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.idx = self.idx.saturating_sub(1);
                }
                KeyCode::Char('y') => {
                    state.settings.syntax_highlight = !state.settings.syntax_highlight;
                    let _ = state.settings.save();
                }
                _ => {}
            }
        }
        Transition::None
    }
}

struct PagerView { data: ViewData }
impl View<AppState> for PagerView {
    fn title(&self) -> String { self.data.title.clone() }
    fn render(&mut self, f: &mut TuiFrame<'_>, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        // Colored footer
        let mut fs = Vec::new();
        fs.push(Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": back  "));
        fs.push(Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": scroll  "));
        fs.push(Span::styled("g/G", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": top/bottom  "));
        fs.push(Span::styled("w", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(format!(": wrap={}  ", if state.settings.wrap_lines { "on" } else { "off" })));
        fs.push(Span::styled("y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(format!(": syn={}  ", if state.settings.syntax_highlight { "on" } else { "off" })));
        fs.push(Span::styled("Tab/p/d", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        fs.push(Span::raw(": switch"));
        f.render_widget(Paragraph::new(Line::from(fs)), chunks[1]);

        let block = Block::default().title(self.title()).borders(Borders::ALL);
        let mut para = Paragraph::new(self.data.content.as_str()).block(block);
        if state.settings.wrap_lines {
            para = para.wrap(ratatui::widgets::Wrap { trim: false });
        }
        para = para.scroll((self.data.scroll_pager, 0));
        f.render_widget(para, chunks[0]);
    }
    fn on_event(&mut self, ev: &Event, state: &mut AppState) -> Transition<AppState> {
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Char('w') => { state.settings.wrap_lines = !state.settings.wrap_lines; let _ = state.settings.save(); }
                KeyCode::Char('q') => return Transition::Back,
                KeyCode::Tab | KeyCode::Char('d') | KeyCode::Char('D') => return Transition::Replace(Box::new(DiffView { data: self.data.clone() })),
                KeyCode::Char('p') => { /* already pager */ }
                KeyCode::Char('j') | KeyCode::Down => { self.data.scroll_pager = self.data.scroll_pager.saturating_add(1); }
                KeyCode::Char('k') | KeyCode::Up => { self.data.scroll_pager = self.data.scroll_pager.saturating_sub(1); }
                KeyCode::Char('g') => { self.data.scroll_pager = 0; }
                KeyCode::Char('G') => { self.data.scroll_pager = u16::MAX; }
                _ => {}
            }
        }
        Transition::None
    }
}

struct DiffView { data: ViewData }
impl View<AppState> for DiffView {
    fn title(&self) -> String { self.data.title.clone() }
    fn render(&mut self, f: &mut TuiFrame<'_>, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        let footer = Paragraph::new(Span::raw("q: back  j/k: scroll  g/G: top/bottom  w: wrap  Tab/p/d: switch"));
        f.render_widget(footer, chunks[1]);

        let block = Block::default().title(self.title()).borders(Borders::ALL);
        // Always color diff headers and +/-; add code syntax when enabled
        let lines = if state.settings.syntax_highlight {
            self.data.lines.clone()
        } else {
            colorize_diff_basic(&self.data.content)
        };
        let mut para = Paragraph::new(lines).block(block);
        if state.settings.wrap_lines {
            para = para.wrap(ratatui::widgets::Wrap { trim: false });
        }
        para = para.scroll((self.data.scroll_diff, 0));
        f.render_widget(para, chunks[0]);
    }
    fn on_event(&mut self, ev: &Event, state: &mut AppState) -> Transition<AppState> {
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Char('w') => { state.settings.wrap_lines = !state.settings.wrap_lines; let _ = state.settings.save(); }
                KeyCode::Char('y') => { state.settings.syntax_highlight = !state.settings.syntax_highlight; let _ = state.settings.save(); }
                KeyCode::Char('q') => return Transition::Back,
                KeyCode::Tab | KeyCode::Char('p') | KeyCode::Char('P') => return Transition::Replace(Box::new(PagerView { data: self.data.clone() })),
                KeyCode::Char('d') => { /* already diff */ }
                KeyCode::Char('j') | KeyCode::Down => { self.data.scroll_diff = self.data.scroll_diff.saturating_add(1); }
                KeyCode::Char('k') | KeyCode::Up => { self.data.scroll_diff = self.data.scroll_diff.saturating_sub(1); }
                KeyCode::Char('g') => { self.data.scroll_diff = 0; }
                KeyCode::Char('G') => { self.data.scroll_diff = u16::MAX; }
                _ => {}
            }
        }
        Transition::None
    }
}
