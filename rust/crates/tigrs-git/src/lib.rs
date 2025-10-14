use anyhow::Result;
use git2::{Oid, Repository, Sort};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,        // short id (7 chars)
    pub full_id: String,   // full 40-char id
    pub summary: String,
    pub author: String,
    pub time: String,
}

pub fn discover_repo(start: Option<&str>) -> Result<Repository> {
    let repo = match start {
        Some(path) => Repository::discover(path)?,
        None => Repository::discover(".")?,
    };
    Ok(repo)
}

pub fn recent_commits(repo: &Repository, limit: usize) -> Result<Vec<CommitInfo>> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

    let mut out = Vec::new();
    for (i, oid) in walk.enumerate() {
        if i >= limit { break; }
        let oid = oid?;
        if let Some(info) = commit_info(repo, oid)? { out.push(info); }
    }
    Ok(out)
}

fn commit_info(repo: &Repository, oid: Oid) -> Result<Option<CommitInfo>> {
    let obj = repo.find_object(oid, None)?;
    let commit = match obj.peel_to_commit() {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let full_id = commit.id().to_string();
    let id = short_id(&commit.id())?;
    let summary = commit.summary().unwrap_or("").to_string();
    let author_sig = commit.author();
    let author = match (author_sig.name(), author_sig.email()) {
        (Some(n), Some(e)) => format!("{} <{}>", n, e),
        (Some(n), None) => n.to_string(),
        _ => String::from("<unknown>"),
    };
    let time = to_rfc3339(commit.time().seconds());

    Ok(Some(CommitInfo { id, full_id, summary, author, time }))
}

fn short_id(oid: &Oid) -> Result<String> {
    let s = oid.to_string();
    Ok(s.chars().take(7).collect())
}

fn to_rfc3339(secs: i64) -> String {
    let dt = OffsetDateTime::from_unix_timestamp(secs).unwrap_or_else(|_| OffsetDateTime::UNIX_EPOCH);
    dt.format(&Rfc3339).unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

pub fn commit_diff_text(repo: &Repository, oid: Oid) -> Result<String> {
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut patch = String::new();
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let origin = line.origin(); // returns a char
        if let Ok(text) = std::str::from_utf8(line.content()) {
            patch.push(origin);
            patch.push_str(text);
        }
        true
    })?;

    let mut out = String::new();
    // Header
    out.push_str(&format!("commit {}\n", commit.id()));
    if let Some(a) = commit.author().name() {
        out.push_str(&format!("Author: {}\n", a));
    }
    out.push_str(&format!("Date:   {}\n\n", to_rfc3339(commit.time().seconds())));
    if let Some(msg) = commit.message() { out.push_str(msg); out.push('\n'); }
    out.push('\n');
    out.push_str(&patch);
    Ok(out)
}

pub fn oid_from_str(repo: &Repository, s: &str) -> Result<Oid> {
    // Accept short or full ids via revparse
    let obj = repo.revparse_single(s)?;
    Ok(obj.id())
}
