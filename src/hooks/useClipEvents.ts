import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useClipsStore } from "../store/clips";
import { useAppStore } from "../store/app";
import type { BootstrapState, ClipAddedPayload } from "../types";

export function useClipEvents() {
  const invalidate = useClipsStore((s) => s.invalidate);
  const markInserted = useClipsStore((s) => s.markInserted);
  const setBootstrap = useAppStore((s) => s.setBootstrap);

  useEffect(() => {
    let active = true;
    const unlisten = listen<ClipAddedPayload>("clip-added", (event) => {
      if (!active) return;
      markInserted(event.payload.clipId);
      void invalidate();
      void invoke<BootstrapState>("get_bootstrap_state").then(setBootstrap).catch(() => undefined);
    });

    return () => {
      active = false;
      unlisten.then((fn) => fn());
    };
  }, [invalidate, markInserted, setBootstrap]);
}
