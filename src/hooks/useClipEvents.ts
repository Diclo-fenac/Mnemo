import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useClipsStore } from "../store/clips";
import type { ClipAddedPayload } from "../types";

export function useClipEvents() {
  const invalidate = useClipsStore((s) => s.invalidate);

  useEffect(() => {
    const unlisten = listen<ClipAddedPayload>("clip-added", () => {
      invalidate();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [invalidate]);
}
