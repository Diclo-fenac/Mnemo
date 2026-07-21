import { FolderOpen, GitBranch, Search, Settings2, Sparkles } from "lucide-react";
import { Link } from "react-router-dom";

type ChatActionGridProps = { onPrompt: (prompt: string) => void; latestSessionId?: string };
const prompts = ["What was I researching yesterday?", "Summarize my latest project context", "Find related clips"];

export function ChatActionGrid({ onPrompt, latestSessionId }: ChatActionGridProps) {
  return <section className="chat-actions-panel" aria-labelledby="chat-actions-title"><header className="chat-section-heading"><div><p className="eyebrow">Start with</p><h2 id="chat-actions-title">Useful shortcuts</h2></div></header><div className="chat-action-groups"><div className="chat-action-group"><span className="chat-action-label">Quick actions</span>{prompts.map((prompt) => <button type="button" key={prompt} onClick={() => onPrompt(prompt)}><Sparkles size={14} />{prompt}</button>)}</div><div className="chat-action-group"><span className="chat-action-label">Workspace</span>{latestSessionId ? <Link to={`/session/${latestSessionId}`}><FolderOpen size={14} />Open latest project</Link> : <Link to="/timeline"><FolderOpen size={14} />Open timeline</Link>}<Link to="/graph"><GitBranch size={14} />Explore memory graph</Link><Link to="/settings"><Settings2 size={14} />Open Settings</Link></div><div className="chat-action-group chat-action-search"><span className="chat-action-label">Search</span><Link to="/search"><Search size={14} />Search all memories</Link></div></div></section>;
}
