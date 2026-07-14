import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, BrainCircuit } from "lucide-react";
import { ClipCard } from "../components/ClipCard";
import type { Clip } from "../types";

export function SessionReconstruction() {
  const { id } = useParams<{ id: string }>();
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    invoke<Clip[]>("get_session_clips", { sessionId: id })
      .then(setClips)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return (
      <section className="page flex items-center justify-center h-full">
        <p className="text-[var(--color-muted)]">Reconstructing session...</p>
      </section>
    );
  }

  return (
    <section className="page">
      <Link to="/" className="inline-flex items-center gap-2 text-sm text-[var(--color-muted)] hover:text-[var(--color-charcoal)] mb-8 transition-colors">
        <ArrowLeft size={16} /> Back to Timeline
      </Link>
      
      <div className="mb-10">
        <p className="eyebrow flex items-center gap-2">
          <BrainCircuit size={14} /> Session Context
        </p>
        <h1 className="page-title">Research Trail</h1>
        <p className="page-copy text-sm">
          {clips.length} memory chunks captured in this focused window.
        </p>
      </div>

      <div className="relative border-l-2 border-[var(--color-soft-border)] ml-4 pl-8 pb-12 space-y-12">
        {clips.map((clip, idx) => (
          <div key={clip.id} className="relative">
            <div className="absolute -left-[41px] top-4 w-4 h-4 rounded-full border-4 border-[var(--color-soft-white)] bg-[var(--color-warm-sand)]" />
            <ClipCard clip={clip} />
            
            {/* If there's context like a page title, render a semantic edge tag */}
            {clip.pageTitle && (
              <div className="mt-3 inline-flex text-xs bg-[var(--color-warm-sand)] px-2 py-1 rounded text-[var(--color-charcoal)] opacity-80">
                Found on: {clip.pageTitle}
              </div>
            )}
          </div>
        ))}
        {clips.length === 0 && (
          <p className="text-[var(--color-muted)]">No memories found for this session.</p>
        )}
      </div>
    </section>
  );
}
