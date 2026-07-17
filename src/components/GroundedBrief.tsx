import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, ThumbsDown, ThumbsUp } from "lucide-react";
import { Link } from "react-router-dom";
import type { SearchResult } from "../types";

export function GroundedBrief({ query, results }: { query: string; results: SearchResult[] }) {
  const [hidden, setHidden] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [edited, setEdited] = useState(false);
  const [note, setNote] = useState(briefText(query, results));
  if (!results.length || hidden) return null;
  const citations = results.slice(0, 3);
  const log = (action: string) => { void invoke("log_search_feedback", { query, queryType: "grounded_brief", resultClipId: citations[0]?.clip.id ?? null, rankPosition: 1, action }); };
  return <section className="grounded-brief"><header><div><p className="eyebrow">Grounded answer</p><h2>What Mnemo found</h2></div><div className="brief-votes"><button onClick={() => log("brief_up")} aria-label="This brief was useful"><ThumbsUp size={15} /></button><button onClick={() => setFeedbackOpen((open) => !open)} aria-expanded={feedbackOpen} aria-label="Improve this brief"><ThumbsDown size={15} /></button></div></header>{edited ? <textarea aria-label="Edit grounded brief" value={note} onChange={(event) => setNote(event.target.value)} onBlur={() => log("brief_edit")} /> : <p>{note}</p>}<div className="brief-citations"><span>Evidence</span>{citations.map((result, index) => <Link key={result.clip.id} to={`/clip/${result.clip.id}`}><b>{index + 1}</b>{result.clip.pageTitle || result.clip.appName || "Captured memory"}</Link>)}</div>{feedbackOpen && <div className="brief-feedback"><button onClick={() => { setEdited(true); setFeedbackOpen(false); }}>Edit</button><button onClick={() => { log("brief_hide"); setHidden(true); }}>Hide</button><button onClick={() => { log("brief_show_less"); setFeedbackOpen(false); }}>Show less like this</button><ChevronDown size={14} /></div>}</section>;
}

function briefText(query: string, results: SearchResult[]) {
  const sources = [...new Set(results.slice(0, 3).map((result) => result.clip.pageTitle || result.clip.appName || "captured memory"))];
  const first = results[0]?.clip.content.replace(/\s+/g, " ").trim().slice(0, 130) ?? "";
  return `Found ${results.length} ${results.length === 1 ? "memory" : "memories"} for “${query}”. Strongest evidence is from ${sources.join(", ")}. Top captured context: ${first}${first.length === 130 ? "…" : ""}`;
}
