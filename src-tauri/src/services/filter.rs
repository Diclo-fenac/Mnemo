use regex::Regex;
use rusqlite::Connection;

use crate::models::filter_rule::FilterRule;

pub struct FilterResult {
    pub blocked: bool,
    pub matched_rule: Option<String>,
}

pub fn load_rules(db: &Connection) -> Vec<FilterRule> {
    let mut stmt = db
        .prepare("SELECT id, rule_type, pattern, action, enabled FROM filter_rules WHERE enabled = 1")
        .unwrap_or_else(|_| panic!("Failed to prepare filter_rules query"));

    stmt.query_map([], |row| {
        Ok(FilterRule {
            id: row.get(0)?,
            rule_type: row.get(1)?,
            pattern: row.get(2)?,
            action: row.get(3)?,
            enabled: row.get::<_, i32>(4)? == 1,
        })
    })
    .unwrap_or_else(|_| panic!("Failed to query filter_rules"))
    .filter_map(|r| r.ok())
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
