import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Check, GitBranch, Keyboard, Search, ShieldCheck, Sparkles } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";
import { ActivityRail } from "../components/ActivityRail";
import { ClipCard } from "../components/ClipCard";
import { useClipsStore } from "../store/clips";
import { useAppStore } from "../store/app";
import { formatDuration, formatSessionRange } from "../lib/presentation";
import type { BootstrapState, CapturePreferences, SessionSummary } from "../types";

export function Memory() {
  const navigate = useNavigate();
  const { clips, loading, fetchClips, latestClipId } = useClipsStore();
  const bootstrap = useAppStore((state) => state.bootstrap);
  const capturePreferences = useAppStore((state) => state.capturePreferences);
  const setBootstrap = useAppStore((state) => state.setBootstrap);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [query, setQuery] = useState("");
  const [onboardingBusy, setOnboardingBusy] = useState(false);

  useEffect(() => { void fetchClips(); }, [fetchClips]);
  useEffect(() => { invoke<SessionSummary[]>("list_sessions", { limit: 6 }).then(setSessions).catch(() => setSessions([])); }, [clips.length]);

  function search(event: React.FormEvent) { event.preventDefault(); const value = query.trim(); if (value) navigate(`/search?q=${encodeURIComponent(value)}`); else navigate("/search"); }
  const resumeSession = sessions[0];
  const connectedCount = sessions.reduce((count, session) => count + Math.max(0, session.clipCount - 1), 0);

  if (loading && !clips.length) return <section className="page"><p className="eyebrow">Your memory</p><h1 className="page-title">Memory</h1><div className="skeleton-stack"><div /><div /><div /></div></section>;
  if (!clips.length && (!bootstrap || !capturePreferences)) return <section className="page"><div className="skeleton-stack"><div /><div /><div /></div></section>;
  if (!clips.length && bootstrap && !bootstrap.onboardingCompleted) return <section className="page onboarding-page"><div className="onboarding-intro"><p className="eyebrow">Welcome to Mnemo</p><h1 className="page-title">Your clipboard, with memory.</h1><p className="page-copy">Mnemo keeps useful copies on this device, then connects them into research you can actually find again.</p></div><div className="onboarding-steps"><article><span><ShieldCheck size={18} /></span><div><b>Capture starts paused</b><p>Nothing enters Mnemo until you explicitly turn capture on. Existing memories stay searchable whenever capture is paused.</p></div><Check size={16} /></article><article><span><Sparkles size={18} /></span><div><b>Preparing local intelligence</b><p>{bootstrap.embeddingStatus === "ready" ? "Your local model is ready to connect related clips." : bootstrap.embeddingStatus === "unavailable" ? "The local model could not be prepared. You can retry from Settings; keyword search still works." : "The local model will download after you continue. Search still works while it loads."}</p></div><Check size={16} /></article><article><span><Keyboard size={18} /></span><div><b>Two shortcuts, one purpose</b><p><kbd>Ctrl/Cmd + Shift + V</kbd> searches your memory. <kbd>Ctrl/Cmd + Shift + M</kbd> pauses or resumes capture.</p></div><Check size={16} /></article></div><button className="primary-button onboarding-start" disabled={onboardingBusy} onClick={() => { setOnboardingBusy(true); void invoke<CapturePreferences>("complete_onboarding", { input: { captureEnabled: false } }).then((saved) => { useAppStore.getState().setCapturePreferences(saved); return invoke<BootstrapState>("get_bootstrap_state"); }).then(setBootstrap).catch(() => setOnboardingBusy(false)); }}>{onboardingBusy ? "Preparing local memory…" : "Continue with capture paused"} <ArrowRight size={15} /></button></section>;
  if (!clips.length) return <section className="page memory-empty"><p className="eyebrow">Your private memory</p><h1 className="page-title">A little context goes a long way.</h1><p className="page-copy">Copy something to begin. Mnemo keeps it on this device, groups related research, and makes it searchable when you need it.</p><div className="memory-empty-steps"><span><b>1</b> Capture is visible and controllable</span><span><b>2</b> Related clips form research sessions</span><span><b>3</b> Ask Mnemo when you need it back</span></div></section>;

  return <section className="memory-dashboard"><main className="memory-main"><header className="memory-header"><p className="eyebrow">Your private memory</p><h1 className="page-title">Pick up where you left off.</h1><form className="memory-ask" onSubmit={search}><Sparkles size={18} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Ask Mnemo about something you copied…" /><button aria-label="Search memory" type="submit"><Search size={17} /></button></form></header><div className="memory-grid"><section className="memory-resume"><div><p className="eyebrow">Continue</p><h2>{resumeSession?.label || "Your next research thread"}</h2><p>{resumeSession ? `${resumeSession.clipCount} captures · ${formatDuration(resumeSession.durationMs)} · ${formatSessionRange(resumeSession.startedAt, resumeSession.endedAt)}` : "Sessions appear as Mnemo observes related captures."}</p></div>{resumeSession ? <Link className="primary-button" to={`/session/${resumeSession.id}`}>Resume <ArrowRight size={15} /></Link> : <Link className="quiet-button" to="/timeline">Open timeline</Link>}</section><section className="memory-connection-card"><GitBranch size={19} /><p className="eyebrow">Connection health</p><strong>{connectedCount}</strong><span>research links forming across {sessions.length} session{sessions.length === 1 ? "" : "s"}</span><Link to="/graph">Explore graph <ArrowRight size={14} /></Link></section></div><section className="memory-recent"><header><div><p className="eyebrow">Recent captures</p><h2>What just entered your memory</h2></div><Link to="/timeline">Open archive <ArrowRight size={14} /></Link></header><div className="clips-list">{clips.slice(0, 4).map((clip) => <ClipCard key={clip.id} clip={clip} density="compact" isNew={clip.id === latestClipId} />)}</div></section></main><ActivityRail clips={clips} /></section>;
}
