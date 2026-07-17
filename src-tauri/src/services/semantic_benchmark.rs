#[derive(Debug, Clone, Copy)]
pub struct BenchmarkPair {
    pub query: &'static str,
    pub candidate: &'static str,
    pub related: bool,
}

pub fn fixture() -> Vec<BenchmarkPair> {
    let related = [
        (
            "docker networking",
            "Docker bridge and host networking configuration",
        ),
        (
            "python async",
            "asyncio TaskGroup and concurrent await patterns",
        ),
        ("react hooks", "useState and useEffect component state"),
        (
            "postgres indexes",
            "PostgreSQL B-tree and composite indexes",
        ),
        ("oauth jwt", "Bearer token and OAuth2 authentication flow"),
        ("rust ownership", "Rust borrow checker and ownership rules"),
        ("sql joins", "SQL JOIN query across related tables"),
        ("github actions", "GitHub Actions workflow YAML"),
        ("css grid", "CSS grid-template-columns responsive layout"),
        ("typescript types", "TypeScript generic type constraints"),
        ("api pagination", "REST API cursor pagination response"),
        (
            "terminal git",
            "git rebase and conflict resolution commands",
        ),
        ("redis cache", "Redis cache invalidation and expiration"),
        (
            "kubernetes pods",
            "Kubernetes deployment and pod scheduling",
        ),
        ("vite build", "Vite production build configuration"),
        ("sqlite fts", "SQLite FTS5 full text search tokenizer"),
        ("memory graph", "semantic graph edges between related clips"),
        ("clipboard privacy", "sensitive clipboard content filtering"),
        ("model migration", "background embedding index migration"),
        (
            "search reranking",
            "cross encoder reranks retrieved candidates",
        ),
        ("tauri command", "Tauri IPC command state management"),
        ("rust threads", "Rust background thread synchronization"),
        ("browser context", "browser URL title clipboard enrichment"),
        (
            "session reconstruction",
            "research session timeline reconstruction",
        ),
        ("embedding quality", "cosine similarity retrieval benchmark"),
    ];
    let unrelated = [
        ("docker networking", "family recipe for roasted vegetables"),
        ("python async", "best hiking trails near the lake"),
        ("react hooks", "monthly electricity bill payment reminder"),
        (
            "postgres indexes",
            "watercolor painting paper recommendations",
        ),
        ("oauth jwt", "new office chair assembly instructions"),
        ("rust ownership", "weekend train timetable information"),
        ("sql joins", "coffee beans with chocolate tasting notes"),
        ("github actions", "indoor plant watering schedule"),
        ("css grid", "travel packing list for winter weather"),
        ("typescript types", "documentary film recommendations"),
        ("api pagination", "home insurance renewal checklist"),
        ("terminal git", "simple dinner ideas with lentils"),
        ("redis cache", "how to clean a wool sweater"),
        ("kubernetes pods", "museum opening hours this weekend"),
        ("vite build", "beginner yoga breathing exercises"),
        ("sqlite fts", "garden soil preparation guide"),
        ("memory graph", "camera lens cleaning instructions"),
        ("clipboard privacy", "healthy breakfast meal plan"),
        ("model migration", "city bus route map"),
        ("search reranking", "movie night snack ideas"),
        ("tauri command", "how to fold a fitted sheet"),
        ("rust threads", "sunset photography locations"),
        ("browser context", "beginner guitar chord progression"),
        ("session reconstruction", "birthday party decoration ideas"),
        ("embedding quality", "how to polish wooden furniture"),
    ];
    related
        .into_iter()
        .map(|(query, candidate)| BenchmarkPair {
            query,
            candidate,
            related: true,
        })
        .chain(
            unrelated
                .into_iter()
                .map(|(query, candidate)| BenchmarkPair {
                    query,
                    candidate,
                    related: false,
                }),
        )
        .collect()
}

pub fn best_threshold(scored: &[(f64, bool)]) -> (f64, f64) {
    let mut best = (0.0, 0.0);
    for step in 0..=100 {
        let threshold = step as f64 / 100.0;
        let (mut true_positive, mut false_positive, mut false_negative) = (0.0, 0.0, 0.0);
        for (score, related) in scored {
            match (*score >= threshold, *related) {
                (true, true) => true_positive += 1.0,
                (true, false) => false_positive += 1.0,
                (false, true) => false_negative += 1.0,
                (false, false) => {}
            }
        }
        let precision = if true_positive + false_positive == 0.0 {
            0.0
        } else {
            true_positive / (true_positive + false_positive)
        };
        let recall = if true_positive + false_negative == 0.0 {
            0.0
        } else {
            true_positive / (true_positive + false_negative)
        };
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        if f1 > best.1 {
            best = (threshold, f1);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_has_balanced_labels() {
        let pairs = fixture();
        assert_eq!(pairs.len(), 50);
        assert_eq!(pairs.iter().filter(|pair| pair.related).count(), 25);
        assert!(pairs
            .iter()
            .all(|pair| !pair.query.is_empty() && !pair.candidate.is_empty()));
    }

    #[test]
    fn threshold_search_returns_best_f1() {
        let (threshold, f1) =
            best_threshold(&[(0.9, true), (0.8, true), (0.2, false), (0.1, false)]);
        assert_eq!(threshold, 0.21);
        assert_eq!(f1, 1.0);
    }
}
