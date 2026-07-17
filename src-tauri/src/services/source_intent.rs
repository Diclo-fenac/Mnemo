#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIntent {
    Docs,
    Github,
    StackOverflow,
    Terminal,
    Editor,
    Other,
}

impl SourceIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Github => "github",
            Self::StackOverflow => "stackoverflow",
            Self::Terminal => "terminal",
            Self::Editor => "editor",
            Self::Other => "other",
        }
    }

    pub fn trust_weight(self) -> f64 {
        match self {
            Self::Docs => 1.0,
            Self::Github => 0.9,
            Self::StackOverflow => 0.8,
            Self::Editor => 0.7,
            Self::Terminal => 0.6,
            Self::Other => 0.5,
        }
    }
}

pub fn detect(app: Option<&str>, url: Option<&str>, title: Option<&str>) -> SourceIntent {
    let app = app.unwrap_or_default().to_lowercase();
    let url = url.unwrap_or_default().to_lowercase();
    let title = title.unwrap_or_default().to_lowercase();

    if url.contains("stackoverflow.com") || url.contains("stackexchange.com") {
        return SourceIntent::StackOverflow;
    }
    if url.contains("github.com") || url.contains("gist.github") {
        return SourceIntent::Github;
    }
    if url.contains("docs.") || url.contains("/docs") || title.contains("documentation") {
        return SourceIntent::Docs;
    }
    if ["code", "cursor", "sublime", "jetbrains", "vim", "neovim"]
        .iter()
        .any(|name| app.contains(name))
    {
        return SourceIntent::Editor;
    }
    if ["terminal", "iterm", "alacritty", "powershell", "console"]
        .iter()
        .any(|name| app.contains(name))
    {
        return SourceIntent::Terminal;
    }
    SourceIntent::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_trusted_sources() {
        assert_eq!(
            detect(None, Some("https://docs.docker.com/network"), None),
            SourceIntent::Docs
        );
        assert_eq!(
            detect(None, Some("https://github.com/a/b"), None),
            SourceIntent::Github
        );
        assert_eq!(detect(Some("VS Code"), None, None), SourceIntent::Editor);
    }
}
