import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Link, useParams } from "react-router-dom";
import hljs from "highlight.js/lib/common";
import { ArrowLeft, Check, ClipboardCopy, Lightbulb, MapPin, Network, Pin, Tag } from "lucide-react";
import type { Clip, ClipContext, RelatedClip } from "../types";
import { timeAgo } from "../lib/presentation";

const MAX_HIGHLIGHT_LENGTH = 20_000;

export function ClipDetail() {
  const { id } = useParams<{ id: string }>();
  const [clip, setClip] = useState<Clip | null>(null);
  const [context, setContext] = useState<ClipContext | null>(null);
  const [related, setRelated] = useState<RelatedClip[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    setLoading(true);
    Promise.all([
      invoke<Clip>("get_clip", { id }),
      invoke<ClipContext>("get_clip_context", { clipId: id }),
      invoke<RelatedClip[]>("get_related_clips", { clipId: id, limit: 5 }),
    ]).then(([nextClip, nextContext, nextRelated]) => { setClip(nextClip); setContext(nextContext); setRelated(nextRelated); }).catch((reason) => setError(reason instanceof Error ? reason.message : "Unable to load this clip.")).finally(() => setLoading(false));
  }, [id]);

  async function copyAgain() {
    if (!clip) return;
    try { await invoke("copy_clip", { id: clip.id }); setCopied(true); window.setTimeout(() => setCopied(false), 2000); }
    catch (reason) { setError(reason instanceof Error ? reason.message : "Copy failed."); }
  }
  async function togglePin() { if (clip) try { setClip({ ...clip, isPinned: await invoke<boolean>("toggle_pin", { id: clip.id }) }); } catch (reason) { setError(reason instanceof Error ? reason.message : "Pin update failed."); } }

  if (loading) return <section className="page"><div className="skeleton-stack detail-skeleton"><div /><div /><div /></div></section>;
  if (!clip) return <section className="page"><p className="eyebrow">Clip detail</p><h1 className="page-title">Clip unavailable</h1>{error && <p className="muted-copy" role="alert">{error}</p>}<Link className="session-link" to="/">Return to timeline</Link></section>;
  const isCode = clip.contentType === "code" || Boolean(clip.language);
  const lines = clip.content.split("\n");
  const displayContent = expanded ? clip.content : lines.slice(0, 10).join("\n");
  let highlighted = "";
  if (isCode && displayContent.length <= MAX_HIGHLIGHT_LENGTH) {
    try { highlighted = clip.language && hljs.getLanguage(clip.language) ? hljs.highlight(displayContent, { language: clip.language }).value : hljs.highlightAuto(displayContent).value; } catch { highlighted = ""; }
  }

  return <section className="page detail-page">
    <Link to="/" className="back-link"><ArrowLeft size={16} /> Back to Timeline</Link>
    {error && <div className="error-banner" role="alert">{error}<button onClick={() => setError(null)}>Dismiss</button></div>}
    <header className="detail-header"><div><p className="eyebrow">Captured memory</p><h1 className="page-title">{clip.pageTitle || clip.appName || "Clipboard item"}</h1><p className="page-copy">Copied {timeAgo(clip.copiedAt)} · {clip.language || clip.contentType}</p></div><div className="detail-actions"><button className="primary-button" onClick={copyAgain}>{copied ? <Check size={16} /> : <ClipboardCopy size={16} />}{copied ? "Copied!" : "Copy again"}</button><button className="quiet-button" onClick={togglePin}><Pin size={15} /> {clip.isPinned ? "Unpin" : "Pin"}</button></div></header>
    <div className="detail-layout">
      <section className="content-panel"><pre className={isCode ? "code-block hljs" : "code-block"} {...(highlighted ? { dangerouslySetInnerHTML: { __html: highlighted } } : {})}>{highlighted ? undefined : displayContent}</pre>{lines.length > 10 && <button className="text-button" onClick={() => setExpanded((value) => !value)}>{expanded ? "Show less" : `Show all ${lines.length} lines`}</button>}</section>
      <aside className="detail-context-rail">
        <section className="context-card"><div className="context-title"><Lightbulb size={17} /><h2>Why did I copy this?</h2></div><dl><div><dt><MapPin size={15} /> Copied from</dt><dd>{context?.source || "Unknown source"} · {clip.appName || "Unknown app"}</dd></div><div><dt><Lightbulb size={15} /> Likely purpose</dt><dd>{context?.likelyPurpose || "General reference"}</dd></div><div><dt><Tag size={15} /> Topic tags</dt><dd>{context?.topicTags?.length ? context.topicTags.map((tag) => <span className="tag" key={tag}>{tag}</span>) : "Still learning this clip"}</dd></div></dl></section>
        <section className="related-section"><div className="context-title"><Network size={17} /><h2>Memory connections</h2></div>{related.length ? <div className="related-list">{related.map((item) => <Link className="related-row" to={`/clip/${item.id}`} key={item.id}><span>{item.content.slice(0, 120)}{item.content.length > 120 ? "..." : ""}</span><small>{Math.round(item.similarity * 100)}% · {timeAgo(item.copiedAt)}</small></Link>)}</div> : <p className="muted-copy">No strong connections yet. Mnemo will link this as your memory grows.</p>}</section>
      </aside>
    </div>
  </section>;
}
