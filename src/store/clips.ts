import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Clip } from "../types";

type ClipsStore = {
  clips: Clip[];
  loading: boolean;
  error: string | null;
  page: number;
  latestClipId: string | null;
  fetchClips: (page?: number) => Promise<void>;
  invalidate: () => Promise<void>;
  removeClip: (id: string) => void;
  updateClip: (id: string, patch: Partial<Clip>) => void;
  markInserted: (id: string) => void;
  reset: () => void;
};

export const useClipsStore = create<ClipsStore>((set, get) => ({
  clips: [],
  loading: false,
  error: null,
  page: 1,
  latestClipId: null,

  fetchClips: async (page = 1) => {
    set({ loading: true, error: null });
    try {
      const clips = await invoke<Clip[]>("list_clips", { page, pageSize: 50 });
      set({ clips, page, loading: false, error: null });
    } catch (e) {
      set({ loading: false, error: e instanceof Error ? e.message : "Unable to load clips." });
    }
  },

  invalidate: async () => {
    await get().fetchClips(1);
  },

  removeClip: (id) => {
    set((s) => ({ clips: s.clips.filter((c) => c.id !== id) }));
  },

  updateClip: (id, patch) => {
    set((s) => ({
      clips: s.clips.map((c) => (c.id === id ? { ...c, ...patch } : c)),
    }));
  },
  markInserted: (id) => {
    set({ latestClipId: id });
    window.setTimeout(() => {
      if (get().latestClipId === id) set({ latestClipId: null });
    }, 1200);
  },
  reset: () => set({ clips: [], loading: false, error: null, page: 1, latestClipId: null }),
}));
