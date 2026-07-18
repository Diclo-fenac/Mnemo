import { create } from "zustand";
import type { BootstrapState, CapturePreferences, UpdateState } from "../types";

type AppStore = {
  bootstrap: BootstrapState | null;
  capturePreferences: CapturePreferences | null;
  updateState: UpdateState;
  setBootstrap: (bootstrap: BootstrapState | null) => void;
  setCapturePreferences: (preferences: CapturePreferences | null) => void;
  setUpdateState: (updateState: UpdateState) => void;
};

export const useAppStore = create<AppStore>((set) => ({
  bootstrap: null,
  capturePreferences: null,
  updateState: { status: "idle" },
  setBootstrap: (bootstrap) => set({ bootstrap }),
  setCapturePreferences: (capturePreferences) => set({ capturePreferences }),
  setUpdateState: (updateState) => set({ updateState }),
}));
