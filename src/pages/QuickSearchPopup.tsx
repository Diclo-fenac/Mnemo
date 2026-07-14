import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Search as SearchIcon, Loader2 } from "lucide-react";
import type { Clip } from "../types";

export function QuickSearchPopup() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    const unlistenBlur = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        getCurrentWindow().hide();
        setQuery("");
        setResults([]);
      }
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        getCurrentWindow().hide();
        setQuery("");
        setResults([]);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      unlistenBlur.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    const timer = setTimeout(async () => {
      setLoading(true);
      try {
        const clips = await invoke<Clip[]>("hybrid_search", { query });
        setResults(clips.slice(0, 5));
        setSelectedIndex(0);
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [query]);

  return (
    <div className="bg-[var(--color-soft-white)] rounded-xl border border-[var(--color-soft-border)] shadow-2xl flex flex-col h-full overflow-hidden" data-tauri-drag-region>
      <div className="relative border-b border-[var(--color-soft-border)] p-4">
        <SearchIcon className="absolute left-6 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" size={18} />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Mnemo Search..."
          className="w-full bg-transparent text-lg text-[var(--color-charcoal)] pl-10 focus:outline-none placeholder-[var(--color-muted)]"
          autoFocus
        />
        {loading && <Loader2 className="absolute right-6 top-1/2 -translate-y-1/2 animate-spin text-[var(--color-muted)]" size={18} />}
      </div>
      <div className="flex-1 overflow-y-auto p-2">
        {results.map((clip, idx) => (
          <div 
            key={clip.id} 
            className={`p-3 rounded-lg mb-1 flex items-start gap-3 cursor-pointer ${idx === selectedIndex ? 'bg-[var(--color-warm-sand)]' : 'hover:bg-[var(--color-warm-sand)]/50'}`}
          >
            <div className="flex-1 overflow-hidden">
              <div className="text-xs text-[var(--color-muted)] mb-1 flex gap-2">
                <span>{clip.appName || clip.contentType}</span>
                <span className="opacity-50">•</span>
                <span className="truncate">{clip.pageTitle || clip.windowTitle}</span>
              </div>
              <div className="text-sm text-[var(--color-charcoal)] truncate">
                {clip.content}
              </div>
            </div>
          </div>
        ))}
        {query && !loading && results.length === 0 && (
          <div className="p-4 text-center text-sm text-[var(--color-muted)]">
            No memories found.
          </div>
        )}
      </div>
    </div>
  );
}
