import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Clip } from "../types";

type ClipsStore = {
  clips: Clip[];
  loading: boolean;
  page: number;
  fetchClips: (page?: number) => Promise<void>;
  invalidate: () => Promise<void>;
  removeClip: (id: string) => void;
  updateClip: (id: string, patch: Partial<Clip>) => void;
};

export const useClipsStore = create<ClipsStore>((set, get) => ({
  clips: [],
  loading: false,
  page: 1,

  fetchClips: async (page = 1) => {
    set({ loading: true });
    try {
      const clips = await invoke<Clip[]>("list_clips", { page, pageSize: 50 });
      set({ clips, page, loading: false });
    } catch (e) {
      console.error("Failed to fetch clips:", e);
      set({ loading: false });
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
}));
