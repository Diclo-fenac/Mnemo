import { create } from "zustand";
import type { BootstrapState } from "../types";

type AppStore = { bootstrap: BootstrapState | null; setBootstrap: (bootstrap: BootstrapState) => void };
export const useAppStore = create<AppStore>((set) => ({ bootstrap: null, setBootstrap: (bootstrap) => set({ bootstrap }) }));
