import { useEffect } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../store/app";

export function useUpdateCheck() {
  const setUpdateState = useAppStore((state) => state.setUpdateState);

  useEffect(() => {
    // Development and the hidden popup should not generate release-feed noise.
    if (import.meta.env.DEV || getCurrentWindow().label !== "main") {
      setUpdateState({ status: "current" });
      return;
    }
    let active = true;
    setUpdateState({ status: "checking" });
    void check()
      .then((update) => {
        if (!active) return;
        setUpdateState(update ? { status: "available", version: update.version, notes: update.body || undefined } : { status: "current" });
      })
      .catch(() => {
        if (active) setUpdateState({ status: "error" });
      });
    return () => { active = false; };
  }, [setUpdateState]);
}
