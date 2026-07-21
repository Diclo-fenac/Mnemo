import { Activity, ArrowUpRight, Clock3, GitBranch, RadioTower } from "lucide-react";
import { Link } from "react-router-dom";
import { formatDuration, formatSessionRange, timeAgo } from "../lib/presentation";
import { deriveSourceProvenance } from "../lib/sourceProvenance";
import type { Clip, SessionSummary } from "../types";

type ChatContextFeedProps = {
  clips: Clip[];
  sessions: SessionSummary[];
  loading: boolean;
  error: string | null;
};

export function ChatContextFeed({ clips, sessions, loading, error }: ChatContextFeedProps) {
  const latestSession = sessions[0];
  return <section className="chat-context" aria-labelledby="chat-context-title">
    <header className="chat-section-heading"><div><p className="eyebrow">Current context</p><h2 id="chat-context-title">Your recent memory</h2></div><span className="chat-context-count">{clips.length} captured</span></header>
    {error && <p className="chat-context-status">Context is unavailable right now. Your chat still works.</p>}
    {loading && !clips.length && <div className="chat-context-loading"><span /><span /><span /></div>}
    {!loading && !error && !clips.length && <div className="chat-context-empty"><Activity size={16} /><span>Capture something to make your research visible here.</span></div>}
    {!!latestSession && <Link className="chat-session-card" to={`/session/${latestSession.id}`}><span className="chat-context-icon"><GitBranch size={16} /></span><span className="chat-session-copy"><strong>{latestSession.label}</strong><small>{latestSession.clipCount} captures · {formatDuration(latestSession.durationMs)} · {formatSessionRange(latestSession.startedAt, latestSession.endedAt)}</small></span><ArrowUpRight size={15} /></Link>}
    {!!clips.length && <div className="chat-context-list">{clips.slice(0, 3).map((clip) => { const provenance = deriveSourceProvenance(clip); return <Link className="chat-context-item" to={`/clip/${clip.id}`} key={clip.id}><span className="chat-context-icon"><RadioTower size={14} /></span><span className="chat-context-copy"><strong>{clip.content.slice(0, 92)}{clip.content.length > 92 ? "…" : ""}</strong><small><span className={provenance.kind === "unavailable" ? "source-unavailable" : undefined}>{provenance.label}</span><span>·</span><span>{timeAgo(clip.copiedAt)}</span></small></span><ArrowUpRight size={14} /></Link>; })}</div>}
    {!!latestSession && <p className="chat-context-footer"><Clock3 size={13} /> Context updates as you capture and connect memories.</p>}
  </section>;
}
