import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, BrainCircuit, Pause, Play, Sparkles } from "lucide-react";
import { ClipCard } from "../components/ClipCard";
import { formatDuration, formatSessionRange, timeAgo } from "../lib/presentation";
import type { SessionReconstruction as Reconstruction } from "../types";

export function SessionReconstruction() {
  const { id } = useParams<{ id: string }>();
  const [data, setData] = useState<Reconstruction | null>(null);
  const [loading, setLoading] = useState(true);
  const [visibleCount, setVisibleCount] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [autoPlay, setAutoPlay] = useState(() => localStorage.getItem("mnemo-reconstruction-autoplay") === "true");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    invoke<Reconstruction>("get_session_reconstruction", { sessionId: id })
      .then((result) => { const shouldPlay = localStorage.getItem("mnemo-reconstruction-autoplay") === "true" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches; setData(result); setVisibleCount(shouldPlay ? 0 : result.clips.length); setPlaying(shouldPlay); })
      .catch((reason) => setError(reason instanceof Error ? reason.message : "Unable to reconstruct this session."))
      .finally(() => setLoading(false));
  }, [id]);

  useEffect(() => {
    if (!playing || !data || visibleCount >= data.clips.length) { if (data && visibleCount >= data.clips.length) setPlaying(false); return; }
    const timer = window.setTimeout(() => setVisibleCount((count) => count + 1), 200);
    return () => window.clearTimeout(timer);
  }, [data, playing, visibleCount]);

  function toggleAutoPlay() { const next = !autoPlay; setAutoPlay(next); localStorage.setItem("mnemo-reconstruction-autoplay", String(next)); }
  function togglePlayback() { if (!data) return; if (visibleCount >= data.clips.length) setVisibleCount(0); setPlaying((current) => !current); }

  if (loading) {
    return (
      <section className="page flex items-center justify-center h-full">
        <div className="skeleton-stack reconstruction-skeleton"><div /><div /><div /></div>
      </section>
    );
  }

  if (!data) return <section className="page"><Link to="/" className="back-link"><ArrowLeft size={16} /> Back to Timeline</Link><p className="eyebrow">Session context</p><h1 className="page-title">Session unavailable</h1><p className="muted-copy" role="alert">{error || "This session could not be loaded."}</p></section>;
  const { session, clips } = data;
  return (
    <section className="page">
      <Link to="/" className="inline-flex items-center gap-2 text-sm text-[var(--color-muted)] hover:text-[var(--color-charcoal)] mb-8 transition-colors">
        <ArrowLeft size={16} /> Back to Timeline
      </Link>
      {error && <div className="error-banner" role="alert">{error}</div>}

      <div className="mb-10">
        <p className="eyebrow flex items-center gap-2">
          <BrainCircuit size={14} /> Session Context
        </p>
        <h1 className="page-title">{session.label}</h1>
        <p className="page-copy">{formatSessionRange(session.startedAt, session.endedAt)} · {formatDuration(session.durationMs)} · {session.clipCount} captured clips</p>
      </div>
      <div className="reconstruction-toolbar"><button className="primary-button" onClick={togglePlayback}>{playing ? <Pause size={16} /> : <Play size={16} />}{playing ? "Pause" : visibleCount >= clips.length ? "Replay" : "Play"}</button><label className="toggle-label"><input type="checkbox" checked={autoPlay} onChange={toggleAutoPlay} /> Auto-play</label></div>
      <section className="reconstruction-meta"><div><span className="meta-label">Sources</span>{data.sourceBreakdown.length ? data.sourceBreakdown.map((source) => <span className="source-chip" key={source.label}>{source.sourceType === "web" ? "Web" : "App"} · {source.label} <b>{source.count}</b></span>) : <span className="muted-copy">No source information</span>}</div><div><span className="meta-label">Topics</span>{session.keyTopics.length ? session.keyTopics.map((topic) => <span className="tag" key={topic}>{topic}</span>) : <span className="muted-copy">Still learning this session</span>}</div></section>
      <div className="relative border-l-2 border-[var(--color-soft-border)] ml-4 pl-8 pb-12 space-y-12">
        {clips.slice(0, visibleCount).map((clip) => (
          <div key={clip.id} className="relative">
            <div className="absolute -left-[41px] top-4 w-4 h-4 rounded-full border-4 border-[var(--color-soft-white)] bg-[var(--color-warm-sand)]" />
            <span className="timeline-time">{timeAgo(clip.copiedAt)}</span>
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
      {data.connections.length > 0 && <section className="connections-panel"><p className="eyebrow"><Sparkles size={13} /> Cross-session memory</p>{data.connections.map((connection) => <p key={connection.clipId}>Connects to “{connection.contentPreview}” from {timeAgo(connection.copiedAt)} ({Math.round(connection.similarity * 100)}% similarity).</p>)}</section>}
    </section>
  );
}
