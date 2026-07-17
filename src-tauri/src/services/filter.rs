use regex::Regex;
use rusqlite::Connection;

use crate::models::filter_rule::FilterRule;

pub struct FilterResult {
    pub blocked: bool,
    pub matched_rule: Option<String>,
}

pub fn load_rules(db: &Connection) -> Vec<FilterRule> {
    let mut stmt =
        match db.prepare("SELECT rule_type, pattern, action FROM filter_rules WHERE enabled = 1") {
            Ok(statement) => statement,
            Err(error) => {
                log::error!("[filter] Could not load rules: {error}");
                return Vec::new();
            }
        };

    let rows = match stmt.query_map([], |row| {
        Ok(FilterRule {
            rule_type: row.get(0)?,
            pattern: row.get(1)?,
            action: row.get(2)?,
        })
    }) {
        Ok(rows) => rows,
        Err(error) => {
            log::error!("[filter] Could not query rules: {error}");
            return Vec::new();
        }
    };
    rows.filter_map(|row| match row {
        Ok(rule) => Some(rule),
        Err(error) => {
            log::warn!("[filter] Ignoring unreadable rule: {error}");
            None
        }
    })
    .collect()
}

pub fn evaluate(content: &str, rules: &[FilterRule]) -> FilterResult {
    for rule in rules {
        if rule.rule_type != "regex" {
            continue;
        }
        match Regex::new(&rule.pattern) {
            Ok(re) => {
                if re.is_match(content) {
                    return FilterResult {
                        blocked: rule.action == "block" || rule.action == "ask",
                        matched_rule: Some(rule.pattern.clone()),
                    };
                }
            }
            Err(_) => {
                eprintln!("[filter] Invalid regex pattern: {}", rule.pattern);
            }
        }
    }

    FilterResult {
        blocked: false,
        matched_rule: None,
    }
}
