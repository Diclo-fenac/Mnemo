#[derive(Debug, Clone)]
pub struct FilterRule {
    pub rule_type: String,
    pub pattern: String,
    pub action: String,
}
