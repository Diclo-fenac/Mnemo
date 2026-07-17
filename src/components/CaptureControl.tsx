import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { EyeOff, Radio, RadioTower } from "lucide-react";
import { useAppStore } from "../store/app";
import type { CapturePreferences } from "../types";

export function CaptureControl() {
  const preferences = useAppStore((state) => state.capturePreferences);
  const setPreferences = useAppStore((state) => state.setCapturePreferences);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!preferences) {
      void invoke<CapturePreferences>("get_capture_preferences")
        .then(setPreferences)
        .catch(() => undefined);
    }
    const unlisten = listen<boolean>("capture-state-changed", (event) => {
      setPreferences(preferences ? { ...preferences, captureEnabled: event.payload } : null);
    });
    return () => { void unlisten.then((fn) => fn()); };
  }, [preferences, setPreferences]);

  const enabled = preferences?.captureEnabled ?? true;
  async function toggle() {
    if (!preferences || saving) return;
    const next = { ...preferences, captureEnabled: !preferences.captureEnabled };
    setPreferences(next);
    setSaving(true);
    try {
      const saved = await invoke<CapturePreferences>("update_capture_preferences", { preferences: next });
      setPreferences(saved);
    } catch {
      setPreferences(preferences);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className={`capture-control ${enabled ? "is-enabled" : "is-disabled"}`}>
      <div className="capture-control-copy">
        <span className="capture-control-icon">{enabled ? <RadioTower size={15} /> : <EyeOff size={15} />}</span>
        <span><strong>Capture {enabled ? "on" : "off"}</strong><small>{enabled ? "Watching clipboard locally" : "Memory search stays available"}</small></span>
      </div>
      <button type="button" onClick={() => { void toggle(); }} disabled={!preferences || saving} aria-pressed={enabled} aria-label="Toggle clipboard capture">
        <span />
      </button>
      <kbd><Radio size={11} /> Shift M</kbd>
    </div>
  );
}
