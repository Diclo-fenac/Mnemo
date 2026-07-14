#[derive(Debug, Clone)]
pub struct FilterRule {
    pub id: i64,
    pub rule_type: String,
    pub pattern: String,
    pub action: String,
    pub enabled: bool,
}
