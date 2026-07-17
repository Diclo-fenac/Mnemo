import { create } from "zustand";
import type { BootstrapState, CapturePreferences } from "../types";

type AppStore = {
  bootstrap: BootstrapState | null;
  capturePreferences: CapturePreferences | null;
  setBootstrap: (bootstrap: BootstrapState | null) => void;
  setCapturePreferences: (preferences: CapturePreferences | null) => void;
};

export const useAppStore = create<AppStore>((set) => ({
  bootstrap: null,
  capturePreferences: null,
  setBootstrap: (bootstrap) => set({ bootstrap }),
  setCapturePreferences: (capturePreferences) => set({ capturePreferences }),
}));
