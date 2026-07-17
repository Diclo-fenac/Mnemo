import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, ChevronRight, ChevronsDownUp, ChevronsUpDown, Clock3, ExternalLink } from "lucide-react";
import { Link } from "react-router-dom";
import { useClipsStore } from "../store/clips";
import { ClipCard } from "../components/ClipCard";
import { formatDuration, formatSessionRange, topicColor } from "../lib/presentation";
import type { SessionSummary } from "../types";
import { BrandMark } from "../components/BrandMark";

export function Timeline() {
  const { clips, loading, error: clipsError, fetchClips, latestClipId } = useClipsStore();
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [sessionClips, setSessionClips] = useState<Record<string, typeof clips>>({});
  const [sessionsLoading, setSessionsLoading] = useState(true);
  const [sessionsError, setSessionsError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  useEffect(() => { void fetchClips(); }, [fetchClips]);

  useEffect(() => {
    setSessionsLoading(true);
    setSessionsError(null);
    invoke<SessionSummary[]>("list_sessions", { limit: 100 })
      .then(setSessions)
      .catch((reason) => setSessionsError(reason instanceof Error ? reason.message : "Unable to load sessions."))
      .finally(() => setSessionsLoading(false));
  }, [clips.length]);

  if ((loading || sessionsLoading) && clips.length === 0) {
    return (
      <section className="page">
        <p className="eyebrow">Your memory</p>
        <h1 className="page-title">Timeline</h1>
        <div className="skeleton-stack" aria-label="Loading your timeline"><div /><div /><div /></div>
      </section>
    );
  }

  if (clips.length === 0) {
    return (
      <section className="page">
        <p className="eyebrow">Your memory</p>
        <h1 className="page-title">What you copied, remembered.</h1>
        <p className="page-copy">
          Mnemo will quietly collect the threads of your research and arrange
          them into sessions you can revisit. Everything stays on your machine.
        </p>
        <div className="stage-card">
          <strong>No clips yet</strong>
          <p>Copy something — text, a URL, a code snippet — and it will appear here instantly.</p>
          <p style={{ fontSize: 13, opacity: 0.7 }}>
            Quick search: <kbd>Ctrl+Shift+V</kbd>
          </p>
        </div>
        <BrandMark className="empty-symbol" size={250} />
      </section>
    );
  }

  const unsorted = clips.filter((clip) => !clip.sessionId);
  const toggle = async (id: string) => {
    if (!sessionClips[id]) {
      try {
        const loaded = await invoke<typeof clips>("get_session_clips", { sessionId: id });
        setSessionClips((current) => ({ ...current, [id]: loaded }));
      } catch (reason) {
        setSessionsError(reason instanceof Error ? reason.message : "Unable to load session clips.");
        return;
      }
    }
    setExpanded((current) => {
    const next = new Set(current);
    if (next.has(id)) next.delete(id); else { next.clear(); next.add(id); }
    return next;
    });
  };
  const expandAll = async () => {
    const missing = sessions.filter((session) => !sessionClips[session.id]);
    try {
      const loaded = await Promise.all(missing.map(async (session) => [session.id, await invoke<typeof clips>("get_session_clips", { sessionId: session.id })] as const));
      if (loaded.length) setSessionClips((current) => ({ ...current, ...Object.fromEntries(loaded) }));
      setExpanded(new Set(sessions.map((session) => session.id)));
    } catch (reason) {
      setSessionsError(reason instanceof Error ? reason.message : "Unable to load all session clips.");
    }
  };

  return (
    <section className="page">
      <p className="eyebrow">Your memory</p>
      <h1 className="page-title">Timeline</h1>
      {(clipsError || sessionsError) && <div className="error-banner" role="alert">{clipsError || sessionsError}<button onClick={() => { void fetchClips(); }}>Retry</button></div>}
      <div className="timeline-heading">
        <p className="page-copy">{clips.length} clip{clips.length !== 1 ? "s" : ""} remembered across {sessions.length} research session{sessions.length !== 1 ? "s" : ""}.</p>
        <div className="timeline-actions">
          <button className="quiet-button" onClick={() => { void expandAll(); }}><ChevronsUpDown size={15} /> Expand all</button>
          <button className="quiet-button" onClick={() => setExpanded(new Set())}><ChevronsDownUp size={15} /> Collapse all</button>
        </div>
      </div>
      <div className="session-list">
        {sessions.map((session) => {
          const isExpanded = expanded.has(session.id);
          const clipsForSession = sessionClips[session.id] ?? [];
          const accent = topicColor(session.keyTopics[0] || session.label);
          return <article className="session-group" style={{ "--session-accent": accent } as React.CSSProperties} key={session.id}>
            <button className="session-toggle" aria-expanded={isExpanded} onClick={() => { void toggle(session.id); }}>
              <span className="session-chevron">{isExpanded ? <ChevronDown size={18} /> : <ChevronRight size={18} />}</span>
              <span className="session-main"><strong>{session.label}</strong><span>{formatSessionRange(session.startedAt, session.endedAt)}</span></span>
              <span className="session-stats"><Clock3 size={14} /> {formatDuration(session.durationMs)} <span>{session.clipCount} clips</span></span>
            </button>
            <div className="session-summary"><p>{session.summary}</p><div>{[...session.keyTopics, ...session.sourceApps].slice(0, 4).map((item) => <span className="tag" key={item}>{item}</span>)}</div></div>
            {isExpanded && <div className="session-expanded">
              {clipsForSession.length ? <div className="clips-list">{clipsForSession.map((clip) => <ClipCard key={clip.id} clip={clip} isNew={clip.id === latestClipId} />)}</div> : <p className="muted-copy">Loading session clips...</p>}
              <Link className="session-link" to={`/session/${session.id}`}>View reconstruction <ExternalLink size={14} /></Link>
            </div>}
          </article>;
        })}
      </div>
      {unsorted.length > 0 && <section className="unsorted-clips"><h2>Unsorted</h2><div className="clips-list">{unsorted.map((clip) => <ClipCard key={clip.id} clip={clip} isNew={clip.id === latestClipId} />)}</div></section>}
      {!sessions.length && clips.length > 0 && <div className="stage-card"><strong>No sessions yet</strong><p>Your research sessions appear automatically after Mnemo has a little more context.</p></div>}
    </section>
  );
}
