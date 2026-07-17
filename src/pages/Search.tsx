import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search as SearchIcon, Loader2 } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { ClipCard } from "../components/ClipCard";
import { GroundedBrief } from "../components/GroundedBrief";
import type { SearchResult } from "../types";

export function Search() {
  const [params] = useSearchParams();
  const requestedQuery = params.get("q") ?? "";
  const [query, setQuery] = useState(requestedQuery);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function search(searchQuery: string) {
    const normalized = searchQuery.trim();
    if (!normalized) return;

    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke<SearchResult[]>("hybrid_search", { query: normalized });
      setResults(searchResults);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Search failed. Try again.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    if (!requestedQuery.trim()) return;
    setQuery(requestedQuery);
    void search(requestedQuery);
  }, [requestedQuery]);

  function handleSearch(event: React.FormEvent) {
    event.preventDefault();
    void search(query);
  }

  return (
    <section className="page flex flex-col h-full">
      <div className="search-header mb-8">
        <form onSubmit={handleSearch} className="relative w-full max-w-2xl">
          <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" size={18} />
          <input
            type="text"
            data-mnemo-search
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search by keywords or meaning (e.g., 'React hooks context')..."
            className="w-full bg-[var(--color-soft-white)] border border-[var(--color-soft-border)] rounded-full py-3 pl-12 pr-4 text-[var(--color-charcoal)] focus:outline-none focus:border-[var(--color-warm-sand)] focus:ring-1 focus:ring-[var(--color-warm-sand)] transition-all"
            autoFocus
          />
          {loading && <Loader2 className="absolute right-4 top-1/2 -translate-y-1/2 animate-spin text-[var(--color-muted)]" size={18} />}
        </form>
      </div>
      
      <div className="flex-1 overflow-y-auto pr-2 pb-8">
        {error && <div className="error-banner" role="alert">{error}</div>}
        {loading && <div className="skeleton-stack search-skeleton"><div /><div /><div /></div>}
        {!loading && results.length > 0 ? (
          <><GroundedBrief query={query} results={results} /><div className="clips-list">
            {results.map((result) => (
              <div className="search-result" key={result.clip.id}>
                <ClipCard density="compact" clip={result.clip} feedbackQuery={query} feedbackRank={results.indexOf(result) + 1} />
                {result.duplicateCount > 0 && <p className="text-xs text-[var(--color-muted)]">Copied {result.duplicateCount + 1} times</p>}
                <p className="mt-1 text-xs text-[var(--color-muted)]">
                  {result.matchReasons.map((reason) => reason.label).join(" · ")}
                </p>
              </div>
            ))}
          </div></>
        ) : (
          !loading && query.length > 0 && (
            <div className="empty-inline">
              <strong>No matching memories</strong><span>Try a source, topic, code identifier, or a shorter phrase for “{query}”.</span>
            </div>
          )
        )}
      </div>
    </section>
  );
}
