import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search as SearchIcon, Loader2 } from "lucide-react";
import { ClipCard } from "../components/ClipCard";
import type { Clip } from "../types";

export function Search() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(false);

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;

    setLoading(true);
    try {
      const clips = await invoke<Clip[]>("hybrid_search", { query });
      setResults(clips);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="page flex flex-col h-full">
      <div className="search-header mb-8">
        <form onSubmit={handleSearch} className="relative w-full max-w-2xl">
          <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" size={18} />
          <input
            type="text"
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
        {results.length > 0 ? (
          <div className="clips-list">
            {results.map((clip) => (
              <ClipCard key={clip.id} clip={clip} />
            ))}
          </div>
        ) : (
          !loading && query.length > 0 && (
            <div className="text-center text-[var(--color-muted)] mt-12">
              No results found for "{query}".
            </div>
          )
        )}
      </div>
    </section>
  );
}
