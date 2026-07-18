import { useEffect } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { useAppStore } from "../store/app";

export function useUpdateCheck() {
  const setUpdateState = useAppStore((state) => state.setUpdateState);

  useEffect(() => {
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
