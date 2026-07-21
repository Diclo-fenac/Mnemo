import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, Cloud, ThumbsDown, ThumbsUp } from "lucide-react";
import { Link } from "react-router-dom";
import type { AiSettings, GroundedAnswer, SearchResult } from "../types";

export function GroundedBrief({ query, results }: { query: string; results: SearchResult[] }) {
  const [hidden, setHidden] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [edited, setEdited] = useState(false);
  const [note, setNote] = useState(briefText(query, results));
  const [answer, setAnswer] = useState<GroundedAnswer | null>(null);
  const [loading, setLoading] = useState(false);
  const [localOnly, setLocalOnly] = useState(false);
  const [aiSettings, setAiSettings] = useState<AiSettings | null>(null);
  const [allowCloud, setAllowCloud] = useState(false);
  const cloudProvider = aiSettings?.provider === "openai" || aiSettings?.provider === "gemini";

  useEffect(() => {
    void invoke<AiSettings>("get_ai_settings").then(setAiSettings).catch(() => setAiSettings(null));
  }, []);

  useEffect(() => {
    let active = true;
    setLoading(true); setHidden(false);
    setAllowCloud(false);
    void invoke<GroundedAnswer>("generate_grounded_answer", { query, clipIds: results.slice(0, 5).map((result) => result.clip.id), localOnly, allowCloud: false })
      .then((value) => { if (active) { setAnswer(value); setNote(value.answer); } })
      .catch(() => { if (active) { setAnswer(null); setNote(briefText(query, results)); } })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [query, results, localOnly]);
  if (!results.length || hidden) return null;
  const citations = results.slice(0, 3);
  const log = (action: string) => { void invoke("log_search_feedback", { query, queryType: "grounded_brief", resultClipId: citations[0]?.clip.id ?? null, rankPosition: 1, action }); };
  const requestCloudAnswer = () => {
    if (!cloudProvider) return;
    setAllowCloud(true);
    setLoading(true);
    void invoke<GroundedAnswer>("generate_grounded_answer", { query, clipIds: results.slice(0, 5).map((result) => result.clip.id), localOnly: false, allowCloud: true })
      .then((value) => { setAnswer(value); setNote(value.answer); })
      .catch(() => undefined)
      .finally(() => setLoading(false));
  };
  const source = answer?.source ?? "local";
  const sourceLabel = source === "local" ? "Local evidence" : source === "ollama" ? "Ollama answer" : source === "openai" ? "OpenAI answer" : source === "gemini" ? "Gemini answer" : `${source} answer`;
  return <section className={`grounded-brief grounded-source-${source}`}><header><div><p className="eyebrow">Grounded answer</p><h2>What Mnemo found</h2></div><div className="brief-votes"><button onClick={() => log("brief_up")} aria-label="This brief was useful"><ThumbsUp size={15} /></button><button onClick={() => setFeedbackOpen((open) => !open)} aria-expanded={feedbackOpen} aria-label="Improve this brief"><ThumbsDown size={15} /></button></div></header>{loading ? <p className="muted-copy">Checking your top memories…</p> : edited ? <textarea aria-label="Edit grounded brief" value={note} onChange={(event) => setNote(event.target.value)} onBlur={() => log("brief_edit")} /> : <p>{note}</p>}{cloudProvider && !localOnly && answer?.source === "local" && <div className="brief-cloud-consent"><Cloud size={15} /><span>Use {aiSettings?.provider === "openai" ? "OpenAI" : "Gemini"} for this answer? Only these cited excerpts will be sent.</span><button type="button" onClick={requestCloudAnswer} disabled={loading || allowCloud}>Allow once</button></div>}<div className="brief-citations"><span className="brief-source-badge"><i aria-hidden="true" />{sourceLabel}</span>{(answer?.citations ?? citations.map((result) => result.clip.id)).map((id, index) => { const result = results.find((item) => item.clip.id === id); return result ? <Link key={id} to={`/clip/${id}`}><b>{index + 1}</b>{result.clip.pageTitle || result.clip.appName || "Captured memory"}</Link> : null; })}</div><button className="brief-local-button" type="button" onClick={() => setLocalOnly(true)} disabled={localOnly}>Use local answer only</button>{feedbackOpen && <div className="brief-feedback"><button onClick={() => { setEdited(true); setFeedbackOpen(false); }}>Edit</button><button onClick={() => { log("brief_hide"); setHidden(true); }}>Hide</button><button onClick={() => { log("brief_show_less"); setFeedbackOpen(false); }}>Show less like this</button><ChevronDown size={14} /></div>}</section>;
}

function briefText(query: string, results: SearchResult[]) {
  const sources = [...new Set(results.slice(0, 3).map((result) => result.clip.pageTitle || result.clip.appName || "captured memory"))];
  const first = results[0]?.clip.content.replace(/\s+/g, " ").trim().slice(0, 130) ?? "";
  return `Found ${results.length} ${results.length === 1 ? "memory" : "memories"} for “${query}”. Strongest evidence is from ${sources.join(", ")}. Top captured context: ${first}${first.length === 130 ? "…" : ""}`;
}
