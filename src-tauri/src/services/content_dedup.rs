use blake3::Hasher;
use rusqlite::{Connection, OptionalExtension};

pub fn normalize_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn content_hash(normalized: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(normalized.as_bytes());
    hasher.finalize().to_hex().to_string()
}

pub fn find_original(conn: &Connection, hash: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT id FROM clips
         WHERE content_hash = ?1 AND is_duplicate = 0
         ORDER BY copied_at ASC LIMIT 1",
        [hash],
        |row| row.get(0),
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_is_stable() {
        assert_eq!(normalize_content("  Hello\n WORLD  "), "hello world");
        assert_eq!(content_hash("hello world").len(), 64);
    }
}
