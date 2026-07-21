import { useEffect, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, CheckCircle2, Database, Download, Eye, Palette, Power, ShieldCheck, Sparkles, Trash2 } from "lucide-react";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Link } from "react-router-dom";
import { useClipsStore } from "../store/clips";
import { useAppStore } from "../store/app";
import type { AiSettings, BootstrapState, CapturePreferences, EmbeddingModelStatus } from "../types";

type Model = { id: string; displayName: string; dimensions: number; sizeMb: number; description: string };
type ModelStatus = EmbeddingModelStatus;
const appearances: CapturePreferences["appearance"][] = ["dark", "system", "light"];

export function Settings() {
  const [clearing, setClearing] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [models, setModels] = useState<Model[]>([]);
  const [selectedModel, setSelectedModel] = useState("bge-small-en-v1.5");
  const [modelStatus, setModelStatus] = useState<ModelStatus | null>(null);
  const [switching, setSwitching] = useState(false);
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [clearOpen, setClearOpen] = useState(false);
  const [clearValue, setClearValue] = useState("");
  const [aiSettings, setAiSettings] = useState<AiSettings | null>(null);
  const [aiKey, setAiKey] = useState("");
  const [aiBusy, setAiBusy] = useState(false);
  const modelMenuRef = useRef<HTMLDivElement>(null);
  const resetClips = useClipsStore((state) => state.reset);
  const setBootstrap = useAppStore((state) => state.setBootstrap);
  const bootstrap = useAppStore((state) => state.bootstrap);
  const preferences = useAppStore((state) => state.capturePreferences);
  const setPreferences = useAppStore((state) => state.setCapturePreferences);
  const updateState = useAppStore((state) => state.updateState);

  useEffect(() => {
    isEnabled().then(setAutostart).catch(() => setNotice("Unable to read launch-on-startup status."));
    invoke<Model[]>("get_supported_embedding_models").then(setModels).catch(() => setNotice("Unable to load embedding models."));
    invoke<string>("get_active_embedding_model").then(setSelectedModel).catch(() => setNotice("Unable to load active embedding model."));
    const refreshModelStatus = () => invoke<ModelStatus>("get_embedding_model_status").then(setModelStatus).catch(() => setNotice("Unable to check local model files."));
    void refreshModelStatus();
    const statusTimer = window.setInterval(() => { void refreshModelStatus(); }, 1000);
    if (!preferences) {
      invoke<CapturePreferences>("get_capture_preferences").then(setPreferences).catch(() => setNotice("Unable to load privacy settings."));
    }
    invoke<AiSettings>("get_ai_settings").then(setAiSettings).catch(() => setNotice("Unable to load AI provider settings."));
    return () => window.clearInterval(statusTimer);
  }, [preferences, setPreferences]);

  useEffect(() => {
    function onPointerDown(event: MouseEvent) {
      if (!modelMenuRef.current?.contains(event.target as Node)) setModelMenuOpen(false);
    }
    window.addEventListener("mousedown", onPointerDown);
    return () => window.removeEventListener("mousedown", onPointerDown);
  }, []);

  async function switchModel(modelId: string) {
    const previousModel = selectedModel;
    setSelectedModel(modelId);
    setModelMenuOpen(false);
    setSwitching(true);
    try {
      await invoke("switch_embedding_model", { modelId });
      setModelStatus({ modelId, cached: false, runtimeStatus: "loading", error: null, cachedModels: [], pendingModel: modelId, pendingState: "migrating" });
      setNotice("Model migration started. Existing search remains available until it completes.");
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Model switch could not start.");
      setSelectedModel(previousModel);
    } finally {
      setSwitching(false);
    }
  }

  async function updatePreferences(changes: Partial<CapturePreferences>) {
    if (!preferences) return;
    const next = { ...preferences, ...changes };
    setPreferences(next);
    try {
      const saved = await invoke<CapturePreferences>("update_capture_preferences", { preferences: next });
      setPreferences(saved);
    } catch (error) {
      setPreferences(preferences);
      setNotice(error instanceof Error ? error.message : "Unable to save settings.");
    }
  }

  async function handleClearDatabase() {
    if (clearValue !== "DELETE") return;
    setClearing(true);
    try {
      await invoke("clear_database");
      resetClips();
      void invoke<BootstrapState>("get_bootstrap_state").then(setBootstrap).catch(() => undefined);
      setClearOpen(false);
      setClearValue("");
      setNotice("All local memories were cleared. Privacy rules were preserved.");
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Database clear failed.");
    } finally {
      setClearing(false);
    }
  }

  async function toggleAutostart() {
    try {
      if (autostart) { await disable(); setAutostart(false); } else { await enable(); setAutostart(true); }
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Unable to change launch-on-startup.");
    }
  }

  const activeModel = models.find((model) => model.id === selectedModel);

  async function retryModel() {
    try {
      await invoke("retry_embedding_model");
      setBootstrap(bootstrap ? { ...bootstrap, embeddingStatus: "loading" } : null);
      setNotice("Model preparation restarted. Keyword search remains available.");
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Unable to retry model preparation.");
    }
  }

  async function saveAiSettings() {
    if (!aiSettings) return;
    setAiBusy(true);
    try {
      const saved = await invoke<AiSettings>("update_ai_settings", { input: { provider: aiSettings.provider, model: aiSettings.model, ollamaUrl: aiSettings.ollamaUrl, apiKey: aiKey || null, clearApiKey: false, cloudConsent: aiSettings.cloudConsent } });
      setAiSettings(saved); setAiKey(""); setNotice("AI provider settings saved.");
    } catch (error) { setNotice(error instanceof Error ? error.message : "Unable to save AI provider settings."); }
    finally { setAiBusy(false); }
  }

  async function testAiProvider() {
    setAiBusy(true);
    try { await testAndSaveAiSettings(); await invoke("test_ai_provider"); setNotice("AI provider connection is working."); }
    catch (error) { setNotice(error instanceof Error ? error.message : "AI provider connection failed."); }
    finally { setAiBusy(false); }
  }

  async function testAndSaveAiSettings() {
    if (!aiSettings) return;
    const saved = await invoke<AiSettings>("update_ai_settings", { input: { provider: aiSettings.provider, model: aiSettings.model, ollamaUrl: aiSettings.ollamaUrl, apiKey: aiKey || null, clearApiKey: false, cloudConsent: aiSettings.cloudConsent } });
    setAiSettings(saved); setAiKey("");
  }

  async function clearAiKey() {
    if (!aiSettings) return;
    setAiBusy(true);
    try { const saved = await invoke<AiSettings>("update_ai_settings", { input: { provider: aiSettings.provider, model: aiSettings.model, ollamaUrl: aiSettings.ollamaUrl, apiKey: null, clearApiKey: true, cloudConsent: false } }); setAiSettings(saved); setAiKey(""); setNotice("AI API key cleared."); }
    catch (error) { setNotice(error instanceof Error ? error.message : "Unable to clear AI API key."); }
    finally { setAiBusy(false); }
  }

  return (
    <section className="page settings-page">
      <header className="settings-header">
        <p className="eyebrow">Local controls</p>
        <h1 className="page-title">Settings</h1>
        <p className="page-copy">Control what Mnemo remembers, how long it stays, and how it looks.</p>
      </header>

      {notice && <div className="error-banner" role="status">{notice}<button onClick={() => setNotice(null)}>Dismiss</button></div>}

      <div className="settings-stack">
        <section className="settings-panel">
          <div className="settings-panel-heading"><Sparkles size={18} /><div><h2>AI answer provider</h2><p>Optional grounded answers use only your top five local search matches.</p></div></div>
          {aiSettings && <>
            <div className="ai-provider-grid"><label>Provider<select value={aiSettings.provider} onChange={(event) => { const provider = event.target.value as AiSettings["provider"]; const defaults = { none: "", ollama: "llama3.2:3b", openai: "gpt-4o-mini", gemini: "gemini-2.0-flash" }; setAiSettings({ ...aiSettings, provider, model: defaults[provider] }); }}><option value="none">Local only</option><option value="ollama">Ollama</option><option value="openai">OpenAI</option><option value="gemini">Gemini</option></select></label><label>Model<input value={aiSettings.model} onChange={(event) => setAiSettings({ ...aiSettings, model: event.target.value })} /></label></div>
            {aiSettings.provider === "ollama" && <label className="ai-field">Ollama URL<input value={aiSettings.ollamaUrl} onChange={(event) => setAiSettings({ ...aiSettings, ollamaUrl: event.target.value })} /></label>}
            {aiSettings.provider !== "none" && <label className="ai-field">API key{aiSettings.hasApiKey && <small>Key saved. Leave blank to keep it.</small>}<input type="password" autoComplete="off" value={aiKey} onChange={(event) => setAiKey(event.target.value)} placeholder={aiSettings.hasApiKey ? "Saved locally" : "Enter API key"} /></label>}
            {(aiSettings.provider === "openai" || aiSettings.provider === "gemini") && <label className="ai-consent"><input type="checkbox" checked={aiSettings.cloudConsent} onChange={(event) => setAiSettings({ ...aiSettings, cloudConsent: event.target.checked })} /> I understand selected clip excerpts will be sent to {aiSettings.provider === "openai" ? "OpenAI" : "Google Gemini"}. API keys are stored in local SQLite settings.</label>}
            <div className="ai-actions"><button type="button" className="quiet-button" disabled={aiBusy} onClick={() => { void saveAiSettings(); }}>Save provider</button>{aiSettings.provider !== "none" && <button type="button" className="quiet-button" disabled={aiBusy} onClick={() => { void testAiProvider(); }}>Test connection</button>}{aiSettings.hasApiKey && <button type="button" className="text-button" disabled={aiBusy} onClick={() => { void clearAiKey(); }}>Clear key</button>}</div>
            <p className="settings-progress">Provider requests never change capture, sessions, tags, or graph data. Local-only search always remains available.</p>
          </>}
        </section>
        <section className="settings-panel">
          <div className="settings-panel-heading"><ShieldCheck size={18} /><div><h2>Privacy & capture</h2><p>Capture is always local and can be paused at any time.</p></div></div>
          <SettingRow title="Clipboard capture" description={preferences?.captureEnabled ? "Mnemo is watching your clipboard locally." : "Capture is paused. Existing memories remain searchable."}>
            <Toggle checked={preferences?.captureEnabled ?? false} disabled={!preferences} onChange={(captureEnabled) => { void updatePreferences({ captureEnabled }); }} />
          </SettingRow>
          <SettingRow title="Browser context" description="Attach verified page titles and URLs only when the optional Mnemo Context Bridge extension sends them.">
            <Toggle checked={preferences?.browserContextEnabled ?? false} disabled={!preferences} onChange={(browserContextEnabled) => { void updatePreferences({ browserContextEnabled }); }} />
          </SettingRow>
          <div className="browser-context-guide"><strong>Optional browser setup</strong><p>Chrome beta users install the unlisted Web Store package. Firefox beta users load the temporary `.xpi` package from the latest GitHub Release; Firefox removes temporary extensions after restart.</p><a href="https://github.com/Diclo-fenac/Mnemo/releases/latest" target="_blank" rel="noreferrer">Open beta release assets</a></div>
          <SettingRow title="Capture shortcut" description="Toggle capture without opening Mnemo."><kbd className="shortcut-key">Ctrl/Cmd + Shift + M</kbd></SettingRow>
        </section>

        <section className="settings-panel">
          <div className="settings-panel-heading"><Palette size={18} /><div><h2>Appearance</h2><p>Dark is the default. System follows your desktop preference.</p></div></div>
          <div className="appearance-options" role="radiogroup" aria-label="Appearance">
            {appearances.map((appearance) => { const currentAppearance = preferences?.appearance ?? "dark"; return <button key={appearance} type="button" role="radio" aria-checked={currentAppearance === appearance} className={currentAppearance === appearance ? "active" : ""} onClick={() => { void updatePreferences({ appearance }); }}>{appearance}</button>; })}
          </div>
        </section>

        <section className="settings-panel">
          <div className="settings-panel-heading"><Database size={18} /><div><h2>Semantic model</h2><p>Model changes re-embed clips in the background. Existing search remains available during migration.</p></div></div>
          <div className="model-select" ref={modelMenuRef}>
            <button type="button" className="model-select-trigger" aria-expanded={modelMenuOpen} onClick={() => setModelMenuOpen((open) => !open)} disabled={switching}>
              <span><strong>{activeModel?.displayName ?? "Loading model"}</strong><small>{activeModel ? `${activeModel.dimensions} dimensions · ~${activeModel.sizeMb}MB` : ""}</small></span>
            </button>
            {modelMenuOpen && <div className="model-select-menu" role="listbox">{models.map((model) => <button key={model.id} type="button" role="option" aria-selected={model.id === selectedModel} onClick={() => { void switchModel(model.id); }}><span><strong>{model.displayName}</strong><small>{model.description}</small></span><em>{model.dimensions}d · {model.sizeMb}MB</em></button>)}</div>}
          </div>
          {modelStatus && <div className={`model-status ${modelStatus.cached && modelStatus.runtimeStatus === "ready" ? "ready" : modelStatus.runtimeStatus === "unavailable" ? "unavailable" : "pending"}`} role="status">
            {modelStatus.cached && modelStatus.runtimeStatus === "ready" ? <CheckCircle2 size={15} /> : modelStatus.runtimeStatus === "loading" ? <Download size={15} /> : <AlertTriangle size={15} />}
            <span>{modelStatus.cached && modelStatus.runtimeStatus === "ready" ? "Semantic model ready locally" : modelStatus.runtimeStatus === "loading" ? (modelStatus.cached ? "Semantic model loading" : "Downloading semantic model…") : modelStatus.runtimeStatus === "deferred" && modelStatus.cached ? "Semantic model installed; waiting for onboarding" : modelStatus.cached ? "Semantic model installed but unavailable" : "Semantic model not installed"}</span>
          </div>}
          {modelStatus?.pendingModel && <p className="settings-progress">Migration pending: {modelStatus.pendingModel} ({modelStatus.pendingState ?? "working"}). The active model remains searchable until the swap completes.</p>}
          {modelStatus && modelStatus.cachedModels.length > 0 && <p className="settings-progress">Detected locally: {modelStatus.cachedModels.join(", ")}. Mnemo uses the active model above and keeps other cached models available for switching.</p>}
          {modelStatus?.error && <p className="model-error" role="alert">Model diagnostic: {modelStatus.error}</p>}
          {switching && <p className="settings-progress">Preparing model and re-embedding memories…</p>}
          {bootstrap?.embeddingStatus === "unavailable" && <div className="model-retry"><p className="settings-progress">The local model is unavailable. Keyword search still works.</p><button type="button" className="quiet-button" onClick={() => { void retryModel(); }}>Retry model preparation</button></div>}
        </section>

        <section className="settings-panel">
          <div className="settings-panel-heading"><Power size={18} /><div><h2>System & retention</h2><p>Local storage rules that keep the memory lightweight.</p></div></div>
          <SettingRow title="Launch on startup" description="Start Mnemo in the background when you log in."><Toggle checked={autostart} onChange={() => { void toggleAutostart(); }} /></SettingRow>
          <SettingRow title="Auto-delete" description="Never is the default. Set a number of days only if you want Mnemo to clean up unpinned clips automatically."><label className="retention-input"><input value={preferences?.autoDeleteDays ?? ""} type="number" min="1" max="3650" disabled={!preferences} placeholder="Never" onChange={(event) => { const value = event.target.value; void updatePreferences({ autoDeleteDays: value ? Number(value) : null }); }} /><span>days</span></label></SettingRow>
        </section>

        <section className="settings-panel local-data-panel">
          <div className="settings-panel-heading"><Trash2 size={18} /><div><h2>Local data</h2><p>Mnemo stores memories on this device only.</p></div></div>
          {!clearOpen ? <SettingRow title="Clear all memories" description="Permanently remove clips, sessions, embeddings, and local search feedback."><button type="button" className="destructive-button" onClick={() => setClearOpen(true)}><AlertTriangle size={15} /> Review deletion</button></SettingRow> : <div className="clear-confirmation"><div><strong>Delete all local memories?</strong><p>This cannot be undone. Type <code>DELETE</code> to enable the final action.</p></div><input value={clearValue} onChange={(event) => setClearValue(event.target.value)} placeholder="DELETE" aria-label="Type DELETE to confirm" autoFocus /><div className="clear-actions"><button type="button" className="quiet-button" onClick={() => { setClearOpen(false); setClearValue(""); }}>Cancel</button><button type="button" className="destructive-button" disabled={clearValue !== "DELETE" || clearing} onClick={() => { void handleClearDatabase(); }}>{clearing ? "Clearing…" : "Delete all data"}</button></div></div>}
        </section>

        <section className="settings-panel settings-advanced">
          <div className="settings-panel-heading"><Eye size={18} /><div><h2>Advanced diagnostics</h2><p>Review local search quality, embedding coverage, and source distribution.</p></div></div>
          <Link className="quiet-button" to="/quality">Open diagnostics</Link>
        </section>

        <section className="settings-panel settings-advanced">
          <div className="settings-panel-heading"><Power size={18} /><div><h2>Mnemo updates</h2><p>Signed update checks run quietly against the Mnemo GitHub Release feed.</p></div></div>
          <p className="settings-progress">{updateState.status === "checking" ? "Checking for updates…" : updateState.status === "available" ? `Version ${updateState.version} is available. Restart Mnemo after installing the release manually.` : updateState.status === "current" ? "You are running the latest available beta." : updateState.status === "error" ? "Update check unavailable. Mnemo remains fully usable offline." : "Update checks will run after startup."}</p>
        </section>
      </div>
    </section>
  );
}

function SettingRow({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return <div className="setting-row"><div><strong>{title}</strong><p>{description}</p></div>{children}</div>;
}

function Toggle({ checked, disabled, onChange }: { checked: boolean; disabled?: boolean; onChange: (checked: boolean) => void }) {
  return <button type="button" className={`setting-toggle ${checked ? "on" : ""}`} role="switch" aria-checked={checked} disabled={disabled} onClick={() => onChange(!checked)}><span /></button>;
}
