import { Check, Copy, ExternalLink, Pin, RadioTower } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Link } from "react-router-dom";
import { useState } from "react";
import { timeAgo } from "../lib/presentation";
import { deriveSourceProvenance } from "../lib/sourceProvenance";
import { useClipsStore } from "../store/clips";
import type { Clip } from "../types";

export function ActivityRail({ clips }: { clips: Clip[] }) {
  return <aside className="activity-rail">
    <header><div><p className="eyebrow">Live activity</p><h2><RadioTower size={16} /> Captured locally</h2></div><span>{clips.length} recent</span></header>
    <div className="activity-list">{clips.length ? clips.slice(0, 6).map((clip) => <ActivityItem key={clip.id} clip={clip} />) : <div className="activity-empty">Capture something to see it arrive here.</div>}</div>
  </aside>;
}

function ActivityItem({ clip }: { clip: Clip }) {
  const updateClip = useClipsStore((state) => state.updateClip);
  const [copied, setCopied] = useState(false);
  const [busy, setBusy] = useState(false);
  const provenance = deriveSourceProvenance(clip);
  async function copy() { setBusy(true); try { await invoke("copy_clip", { id: clip.id }); setCopied(true); window.setTimeout(() => setCopied(false), 1600); } finally { setBusy(false); } }
  async function pin() { setBusy(true); try { const isPinned = await invoke<boolean>("toggle_pin", { id: clip.id }); updateClip(clip.id, { isPinned }); } finally { setBusy(false); } }
  return <article className="activity-item"><div className="activity-item-meta"><span className={provenance.kind === "unavailable" ? "source-unavailable" : undefined} title={provenance.detail}>{provenance.label}</span><time>{timeAgo(clip.copiedAt)}</time></div><Link to={`/clip/${clip.id}`}>{clip.content.slice(0, 105)}{clip.content.length > 105 ? "…" : ""}</Link><div className="activity-item-actions"><span>{clip.sessionId ? "Research session" : "Unsorted"}</span><div><Link to={`/clip/${clip.id}`} aria-label="Open clip"><ExternalLink size={13} /></Link><button disabled={busy} onClick={() => { void copy(); }} aria-label="Copy clip">{copied ? <Check size={13} /> : <Copy size={13} />}</button><button disabled={busy} onClick={() => { void pin(); }} aria-label={clip.isPinned ? "Unpin clip" : "Pin clip"}><Pin size={13} /></button></div></div></article>;
}
