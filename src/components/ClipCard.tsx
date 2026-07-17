import { Check, Pin, Copy, Trash2, Globe, Code, FileText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useState } from "react";
import { useClipsStore } from "../store/clips";
import type { Clip } from "../types";
import { timeAgo } from "../lib/presentation";

const typeIcons = { url: Globe, code: Code, text: FileText } as const;

export function ClipCard({ clip, feedbackQuery, feedbackRank, density = "reading", isNew = false }: {
  clip: Clip;
  feedbackQuery?: string;
  feedbackRank?: number;
  density?: "reading" | "compact";
  isNew?: boolean;
}) {
  const { removeClip, updateClip } = useClipsStore();
  const navigate = useNavigate();
  const [copied, setCopied] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const TypeIcon = typeIcons[clip.contentType as keyof typeof typeIcons] ?? FileText;

  const preview =
    clip.content.length > 200
      ? clip.content.slice(0, 200) + "…"
      : clip.content;

  async function handleCopy() {
    setBusy(true); setActionError(null);
    try {
      await invoke("copy_clip", { id: clip.id });
      setCopied(true); window.setTimeout(() => setCopied(false), 2000);
      if (feedbackQuery) await invoke("log_copy_again", { query: feedbackQuery, resultClipId: clip.id, rankPosition: feedbackRank });
    } catch (reason) { setActionError(reason instanceof Error ? reason.message : "Copy failed."); } finally { setBusy(false); }
  }

  async function handlePin() {
    setBusy(true); setActionError(null);
    try { const newState = await invoke<boolean>("toggle_pin", { id: clip.id }); updateClip(clip.id, { isPinned: newState }); } catch (reason) { setActionError(reason instanceof Error ? reason.message : "Pin update failed."); } finally { setBusy(false); }
  }

  async function handleDelete() {
    setBusy(true); setActionError(null);
    try { await invoke("delete_clip", { id: clip.id }); removeClip(clip.id); } catch (reason) { setActionError(reason instanceof Error ? reason.message : "Delete failed."); } finally { setBusy(false); }
  }

  return (
    <article className={`clip-card clip-card-${density} ${isNew ? "clip-card-new" : ""}`} role="button" tabIndex={0} onClick={() => navigate(`/clip/${clip.id}`)} onKeyDown={(event) => {
      if (event.key === "Enter" || event.key === " ") { event.preventDefault(); navigate(`/clip/${clip.id}`); }
    }}>
      <div className="clip-header">
        <div className="clip-meta">
          <TypeIcon size={14} />
          {clip.appName && <span>{clip.appName}</span>}
          {clip.language && <span className="clip-lang">{clip.language}</span>}
          <span className="clip-time">{timeAgo(clip.copiedAt)}</span>
        </div>
        <div className="clip-actions">
          <button disabled={busy} onClick={(event) => { event.stopPropagation(); void handleCopy(); }} title={copied ? "Copied" : "Copy again"} aria-label="Copy clip">
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
          <button
            disabled={busy} onClick={(event) => { event.stopPropagation(); void handlePin(); }}
            title={clip.isPinned ? "Unpin" : "Pin"}
            aria-label={clip.isPinned ? "Unpin clip" : "Pin clip"}
            className={clip.isPinned ? "pin-active" : ""}
          >
            <Pin size={14} />
          </button>
          <button disabled={busy} onClick={(event) => { event.stopPropagation(); void handleDelete(); }} title="Delete" aria-label="Delete clip">
            <Trash2 size={14} />
          </button>
        </div>
      </div>
      <pre className="clip-content">{preview}</pre>
      {actionError && <p className="card-error" role="alert">{actionError}</p>}
    </article>
  );
}
