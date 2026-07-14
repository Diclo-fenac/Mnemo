import { Pin, Copy, Trash2, Globe, Code, FileText } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useClipsStore } from "../store/clips";
import type { Clip } from "../types";

const typeIcons = { url: Globe, code: Code, text: FileText } as const;

function timeAgo(epoch: number): string {
  const diff = Math.floor(Date.now() / 1000 - epoch);
  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

export function ClipCard({ clip }: { clip: Clip }) {
  const { removeClip, updateClip } = useClipsStore();
  const TypeIcon = typeIcons[clip.contentType as keyof typeof typeIcons] ?? FileText;

  const preview =
    clip.content.length > 200
      ? clip.content.slice(0, 200) + "…"
      : clip.content;

  async function handleCopy() {
    await invoke("copy_clip", { id: clip.id });
  }

  async function handlePin() {
    const newState = await invoke<boolean>("toggle_pin", { id: clip.id });
    updateClip(clip.id, { isPinned: newState });
  }

  async function handleDelete() {
    await invoke("delete_clip", { id: clip.id });
    removeClip(clip.id);
  }

  return (
    <article className="clip-card">
      <div className="clip-header">
        <div className="clip-meta">
          <TypeIcon size={14} />
          {clip.appName && <span>{clip.appName}</span>}
          {clip.language && <span className="clip-lang">{clip.language}</span>}
          <span className="clip-time">{timeAgo(clip.copiedAt)}</span>
        </div>
        <div className="clip-actions">
          <button onClick={handleCopy} title="Copy" aria-label="Copy clip">
            <Copy size={14} />
          </button>
          <button
            onClick={handlePin}
            title={clip.isPinned ? "Unpin" : "Pin"}
            aria-label={clip.isPinned ? "Unpin clip" : "Pin clip"}
            className={clip.isPinned ? "pin-active" : ""}
          >
            <Pin size={14} />
          </button>
          <button onClick={handleDelete} title="Delete" aria-label="Delete clip">
            <Trash2 size={14} />
          </button>
        </div>
      </div>
      <pre className="clip-content">{preview}</pre>
    </article>
  );
}
