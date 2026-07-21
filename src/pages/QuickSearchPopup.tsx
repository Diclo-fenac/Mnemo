import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Search as SearchIcon, Loader2 } from "lucide-react";
import type { Clip, SearchResult } from "../types";
import { timeAgo } from "../lib/presentation";
import { BrandMark } from "../components/BrandMark";
import { deriveSourceProvenance } from "../lib/sourceProvenance";

export function QuickSearchPopup() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [recent, setRecent] = useState<Clip[]>([]);
  const [error, setError] = useState<string | null>(null);
  const requestId = useRef(0);

  useEffect(() => {
    const loadRecent = () => invoke<Clip[]>("list_clips", { page: 1, pageSize: 5 }).then(setRecent).catch(() => setError("Unable to load recent clips."));
    void loadRecent();
    const unlistenBlur = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        getCurrentWindow().hide();
        setQuery("");
        setResults([]);
      } else {
        void loadRecent();
      }
    });
    return () => {
      unlistenBlur.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    const items = query.trim() ? results.map((result) => result.clip) : recent;
    const copySelected = async () => {
      if (!items[selectedIndex]) return;
      try { await invoke("copy_clip", { id: items[selectedIndex].id }); getCurrentWindow().hide(); setQuery(""); setResults([]); }
      catch (reason) { setError(reason instanceof Error ? reason.message : "Copy failed."); }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") { getCurrentWindow().hide(); setQuery(""); setResults([]); }
      if (event.key === "ArrowDown" && items.length) { event.preventDefault(); setSelectedIndex((current) => Math.min(current + 1, items.length - 1)); }
      if (event.key === "ArrowUp" && items.length) { event.preventDefault(); setSelectedIndex((current) => Math.max(current - 1, 0)); }
      if (event.key === "Enter" && items[selectedIndex]) { event.preventDefault(); void copySelected(); }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [query, recent, results, selectedIndex]);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setSelectedIndex(0);
      return;
    }

    const currentRequest = ++requestId.current;
    const timer = setTimeout(async () => {
      setLoading(true);
      setError(null);
      try {
        const searchResults = await invoke<SearchResult[]>("hybrid_search", { query });
        if (currentRequest !== requestId.current) return;
        setResults(searchResults.slice(0, 5));
        setSelectedIndex(0);
      } catch (reason) {
        if (currentRequest === requestId.current) setError(reason instanceof Error ? reason.message : "Search failed.");
      } finally {
        if (currentRequest === requestId.current) setLoading(false);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [query]);

  const items = query.trim() ? results.map((result) => result.clip) : recent;
  return (
    <div className="quick-search-shell">
      <div className="quick-search-input-wrap">
        <BrandMark size={21} className="quick-search-mark" />
        <SearchIcon className="absolute left-6 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" size={18} />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search your memory..."
          className="w-full bg-transparent text-lg text-[var(--color-charcoal)] pl-10 focus:outline-none placeholder-[var(--color-muted)]"
          autoFocus
        />
        {loading && <Loader2 className="absolute right-6 top-1/2 -translate-y-1/2 animate-spin text-[var(--color-muted)]" size={18} />}
      </div>
      <div className="quick-search-results">
        {!query && <p className="popup-section-label">Recent clips</p>}
        {items.map((clip, idx) => {
          return (
          <div 
            key={clip.id} 
            onMouseEnter={() => setSelectedIndex(idx)}
            onClick={() => { setSelectedIndex(idx); void invoke("copy_clip", { id: clip.id }).then(() => getCurrentWindow().hide()).catch((reason) => setError(reason instanceof Error ? reason.message : "Copy failed.")); }}
            className={`popup-result ${idx === selectedIndex ? 'selected' : ''}`}
          >
            <div className="flex-1 overflow-hidden">
              <div className="text-xs text-[var(--color-muted)] mb-1 flex gap-2">
                <span>{deriveSourceProvenance(clip).label}</span><span>{timeAgo(clip.copiedAt)}</span>
                <span className="opacity-50">•</span>
                <span className="truncate">{deriveSourceProvenance(clip).detail}</span>
              </div>
              <div className="text-sm text-[var(--color-charcoal)] truncate">
                {clip.content}
              </div>
            </div>
          </div>
          );
        })}
        {query && !loading && results.length === 0 && (
          <div className="p-4 text-center text-sm text-[var(--color-muted)]">
            No memories found.
          </div>
        )}
        {error && <div className="popup-error" role="alert">{error}</div>}
      </div>
      <footer className="popup-footer"><span><kbd>↑↓</kbd> navigate</span><span><kbd>↵</kbd> copy</span><span><kbd>esc</kbd> close</span></footer>
    </div>
  );
}
