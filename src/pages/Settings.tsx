import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, Database, Power, ShieldAlert } from "lucide-react";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';

export function Settings() {
  const [clearing, setClearing] = useState(false);
  const [autostart, setAutostart] = useState(false);

  // Initialize autostart state
  useState(() => {
    isEnabled().then(setAutostart).catch(console.error);
  });

  async function handleClearDatabase() {
    if (!window.confirm("Are you sure? This will permanently delete all memories and sessions. This cannot be undone.")) {
      return;
    }

    setClearing(true);
    try {
      await invoke("clear_database");
      alert("Database cleared successfully.");
    } catch (e) {
      console.error(e);
      alert("Failed to clear database.");
    } finally {
      setClearing(false);
    }
  }

  async function toggleAutostart() {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
    } catch (e) {
      console.error("Failed to toggle autostart", e);
    }
  }

  return (
    <section className="page max-w-3xl mx-auto">
      <div className="mb-10">
        <h1 className="page-title">Settings</h1>
        <p className="page-copy">Configure Mnemo's behavior and manage your data.</p>
      </div>

      <div className="space-y-8">
        
        {/* System Settings */}
        <div className="bg-[var(--color-soft-white)] border border-[var(--color-soft-border)] rounded-xl p-6">
          <h2 className="text-lg font-medium text-[var(--color-charcoal)] mb-4 flex items-center gap-2">
            <Power size={18} /> System Integration
          </h2>
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium text-[var(--color-charcoal)]">Launch on Startup</div>
              <div className="text-sm text-[var(--color-muted)] mt-1">Start Mnemo in the background when you log in.</div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" className="sr-only peer" checked={autostart} onChange={toggleAutostart} />
              <div className="w-11 h-6 bg-[var(--color-soft-border)] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-warm-sand)]"></div>
            </label>
          </div>
        </div>

        {/* Data Management */}
        <div className="bg-[var(--color-soft-white)] border border-[var(--color-soft-border)] rounded-xl p-6">
          <h2 className="text-lg font-medium text-[var(--color-charcoal)] mb-4 flex items-center gap-2 text-red-600">
            <Database size={18} /> Danger Zone
          </h2>
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium text-[var(--color-charcoal)]">Clear Database</div>
              <div className="text-sm text-[var(--color-muted)] mt-1">Permanently delete all captured clips, sessions, and embeddings.</div>
            </div>
            <button 
              onClick={handleClearDatabase}
              disabled={clearing}
              className="px-4 py-2 bg-red-100 text-red-700 hover:bg-red-200 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
            >
              {clearing ? "Clearing..." : <><AlertTriangle size={16} /> Delete All Data</>}
            </button>
          </div>
        </div>

      </div>
    </section>
  );
}
