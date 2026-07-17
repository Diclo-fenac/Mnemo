use std::collections::{BTreeSet, HashMap};

use rusqlite::{params, Connection};

const STOP_WORDS: &[&str] = &[
    "a",
    "an",
    "and",
    "as",
    "at",
    "be",
    "by",
    "class",
    "const",
    "def",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "is",
    "it",
    "let",
    "of",
    "on",
    "or",
    "return",
    "the",
    "this",
    "to",
    "true",
    "false",
    "null",
    "undefined",
    "var",
    "with",
];

pub fn refresh_session(conn: &Connection, session_id: &str) -> rusqlite::Result<()> {
    let mut statement = conn.prepare(
        "SELECT content, app_name, source_url FROM clips WHERE session_id = ?1 ORDER BY copied_at ASC",
    )?;
    let rows = statement.query_map([session_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let mut keywords = HashMap::<String, usize>::new();
    let mut apps = BTreeSet::<String>::new();
    let mut domains = BTreeSet::<String>::new();
    let mut count = 0_i64;
    for row in rows {
        let (content, app_name, source_url) = row?;
        count += 1;
        if let Some(app) = app_name.filter(|value| !value.trim().is_empty()) {
            apps.insert(app);
        }
        if let Some(url) = source_url {
            domains.insert(domain(&url));
        }
        for word in content.split(|value: char| !value.is_alphanumeric() && value != '_') {
            let word = word.to_ascii_lowercase();
            if word.len() >= 3
                && !STOP_WORDS.contains(&word.as_str())
                && !word.chars().all(char::is_numeric)
            {
                *keywords.entry(word).or_default() += 1;
            }
        }
    }

    let mut key_topics = keywords.into_iter().collect::<Vec<_>>();
    key_topics.sort_by(|(left_word, left_count), (right_word, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_word.cmp(right_word))
    });
    let key_topics = key_topics
        .into_iter()
        .take(4)
        .map(|(word, _)| word)
        .collect::<Vec<_>>();
    let source_apps = apps.into_iter().collect::<Vec<_>>();
    let source_urls = domains.into_iter().collect::<Vec<_>>();
    let source = source_urls
        .first()
        .cloned()
        .or_else(|| source_apps.first().cloned());
    let topic_label = key_topics
        .iter()
        .take(2)
        .map(|topic| title_case(topic))
        .collect::<Vec<_>>()
        .join(" & ");
    let label = match (source, topic_label.is_empty()) {
        (Some(source), false) => format!("{source}: {topic_label}"),
        (Some(source), true) => format!("{source}: Research"),
        (None, false) => format!("{topic_label} research"),
        (None, true) => "Research session".to_string(),
    };
    let apps_label = if source_apps.is_empty() {
        "Unknown app".to_string()
    } else {
        source_apps.join(", ")
    };
    let topic_summary = if key_topics.is_empty() {
        "general reference".to_string()
    } else {
        key_topics.join(", ")
    };
    conn.execute(
        "UPDATE sessions SET label = ?1, summary = ?2, key_topics = ?3, source_apps = ?4,
         source_urls = ?5, clip_count = ?6 WHERE id = ?7",
        params![
            label,
            format!("{count} clips from {apps_label}. Key topics: {topic_summary}."),
            serde_json::to_string(&key_topics).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&source_apps).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&source_urls).unwrap_or_else(|_| "[]".to_string()),
            count,
            session_id,
        ],
    )?;
    Ok(())
}

fn domain(url: &str) -> String {
    url.split("//")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .trim_start_matches("www.")
        .to_string()
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
        .unwrap_or_default()
}
