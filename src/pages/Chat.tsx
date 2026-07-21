import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowUp, Bot, ExternalLink, RotateCcw } from "lucide-react";
import { Link } from "react-router-dom";
import { ChatActionGrid } from "../components/ChatActionGrid";
import { ChatContextFeed } from "../components/ChatContextFeed";
import { useClipsStore } from "../store/clips";
import type { GroundedAnswer, SearchResult, SessionSummary } from "../types";

type Citation = { id: string; label: string; href?: string };
type ChatMessage = { id: number; role: "user" | "assistant"; text: string; source?: string; citations?: Citation[]; demo?: boolean };

export function Chat() {
  const { clips, loading: clipsLoading, fetchClips } = useClipsStore();
  const [question, setQuestion] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sourceCount, setSourceCount] = useState<number | null>(null);
  const [demoMode, setDemoMode] = useState(false);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [contextLoading, setContextLoading] = useState(true);
  const [contextError, setContextError] = useState<string | null>(null);

  useEffect(() => { void fetchClips(); }, [fetchClips]);
  useEffect(() => {
    setContextLoading(true);
    invoke<SessionSummary[]>("list_sessions", { limit: 6 }).then(setSessions).catch((reason) => setContextError(reason instanceof Error ? reason.message : "Unable to load recent context.")).finally(() => setContextLoading(false));
  }, [clips.length]);

  async function ask(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const query = question.trim();
    if (!query || busy) return;
    setQuestion(""); setError(null); setBusy(true);
    setMessages((current) => [...current, { id: Date.now(), role: "user", text: query, demo: demoMode }]);
    if (demoMode) {
      setSourceCount(3);
      window.setTimeout(() => {
        setMessages((current) => [...current, { id: Date.now() + 1, role: "assistant", text: demoAnswer(query), source: "local", demo: true, citations: demoCitations }]);
        setBusy(false);
      }, 450);
      return;
    }
    try {
      const results = await invoke<SearchResult[]>("hybrid_search", { query });
      setSourceCount(results.length);
      if (!results.length) {
        setMessages((current) => [...current, { id: Date.now() + 1, role: "assistant", text: "I couldn’t find relevant memories for that yet. Try a more specific phrase or capture a little more context first.", source: "local", citations: [] }]);
        return;
      }
      const history = messages.slice(-6).map((message) => `${message.role === "user" ? "User" : "Mnemo"}: ${message.text}`).join("\n");
      const contextualQuery = history ? `Conversation so far:\n${history}\n\nNew question: ${query}` : query;
      const answer = await invoke<GroundedAnswer>("generate_grounded_answer", { query: contextualQuery, clipIds: results.slice(0, 5).map((result) => result.clip.id), localOnly: false, allowCloud: true });
      const citations = (answer.citations ?? []).map((id) => results.find((result) => result.clip.id === id)).filter((result): result is SearchResult => Boolean(result)).map((result) => ({ id: result.clip.id, label: result.clip.pageTitle || result.clip.appName || "Captured memory", href: `/clip/${result.clip.id}` }));
      setMessages((current) => [...current, { id: Date.now() + 1, role: "assistant", text: answer.answer, source: answer.source, citations }]);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Mnemo could not answer that yet.");
    } finally { setBusy(false); }
  }

  function startNewChat() { setQuestion(""); setMessages([]); setError(null); setSourceCount(null); }
  function handleKeyDown(event: React.KeyboardEvent<HTMLTextAreaElement>) { if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); } }
  return <section className="page chat-page">
    <header className="chat-header"><div><p className="eyebrow">Private assistant</p><h1 className="chat-title">Chat with Mnemo</h1><p className="chat-lede">Ask about something you captured, then follow the thread.</p></div><div className="chat-header-actions"><button type="button" className="chat-text-button" onClick={startNewChat}><RotateCcw size={14} /> New chat</button><button type="button" className={`chat-demo-button ${demoMode ? "active" : ""}`} onClick={() => { setDemoMode((enabled) => !enabled); startNewChat(); }}>{demoMode ? "Demo mode on" : "Preview demo"}</button></div></header>
    {!messages.length && <><ChatContextFeed clips={clips} sessions={sessions} loading={contextLoading || clipsLoading} error={contextError} /><ChatActionGrid latestSessionId={sessions[0]?.id} onPrompt={setQuestion} /></>}
    {!!messages.length && <div className="chat-transcript" aria-live="polite">{messages.map((message) => <article className={`chat-message chat-message-${message.role}`} key={message.id}><div className="chat-message-label">{message.role === "user" ? "You" : <><Bot size={13} /> Mnemo</>}</div><p>{message.text}</p>{message.role === "assistant" && <><span className={`chat-source chat-source-${message.source ?? "local"}`}><i />{sourceLabel(message.source)}{message.demo ? " · demo context" : ""}</span>{message.citations?.length ? <div className="chat-citations">{message.citations.map((citation, index) => citation.href ? <Link key={citation.id} to={citation.href}><b>{index + 1}</b>{citation.label}<ExternalLink size={12} /></Link> : <span key={citation.id}><b>{index + 1}</b>{citation.label}</span>)}</div> : <small className="chat-no-sources">No matching sources found</small>}</>}</article>)}{busy && <article className="chat-message chat-message-assistant chat-message-loading"><div className="chat-message-label"><Bot size={13} /> Mnemo</div><p>{demoMode ? "Preparing demo context…" : `Searching ${Math.min(sourceCount ?? 5, 5)} memories…`}</p></article>}</div>}
    {error && <p className="error-banner" role="alert">{error}</p>}
    <form className="chat-composer" onSubmit={ask}><div className="chat-composer-row"><textarea value={question} onChange={(event) => setQuestion(event.target.value)} onKeyDown={handleKeyDown} placeholder="Ask Mnemo…" aria-label="Message Mnemo" rows={1} disabled={busy} /><button type="submit" aria-label="Send message" disabled={!question.trim() || busy}><ArrowUp size={18} /></button></div></form>
  </section>;
}

const demoCitations: Citation[] = [{ id: "demo-session", label: "Research session · local" }, { id: "demo-graph", label: "Memory graph · 3 links" }];
function demoAnswer(query: string) { if (query.toLowerCase().includes("yesterday")) return "You were tracing how Mnemo groups copied research into sessions, then checking how grounded answers cite the original clips. The strongest thread connected the local embedding model, session reconstruction, and graph edges."; if (query.toLowerCase().includes("project")) return "Your latest project context centers on a local-first memory workspace: clipboard capture stays opt-in, evidence remains on-device, and optional providers receive only selected excerpts."; return "I found a related thread across your captured research: local memory capture, semantic connections, and source-backed answers are the recurring theme."; }
function sourceLabel(source?: string) { return source === "gemini" ? "Gemini answer" : source === "openai" ? "OpenAI answer" : source === "ollama" ? "Ollama answer" : "Local evidence"; }
