import { lazy, Suspense, useEffect } from "react";
import { Route, Routes, useNavigate } from "react-router-dom";
import { Sidebar } from "./components/Sidebar";
import { QuickSearchPopup } from "./pages/QuickSearchPopup";
import { useClipEvents } from "./hooks/useClipEvents";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "./store/app";
import { useUpdateCheck } from "./hooks/useUpdateCheck";
import type { BootstrapState, CapturePreferences } from "./types";

const Timeline = lazy(() => import("./pages/Timeline").then((module) => ({ default: module.Timeline })));
const Memory = lazy(() => import("./pages/Memory").then((module) => ({ default: module.Memory })));
const Search = lazy(() => import("./pages/Search").then((module) => ({ default: module.Search })));
const SessionReconstruction = lazy(() => import("./pages/SessionReconstruction").then((module) => ({ default: module.SessionReconstruction })));
const MemoryGraph = lazy(() => import("./pages/MemoryGraph").then((module) => ({ default: module.MemoryGraph })));
const Chat = lazy(() => import("./pages/Chat").then((module) => ({ default: module.Chat })));
const ClipDetail = lazy(() => import("./pages/ClipDetail").then((module) => ({ default: module.ClipDetail })));
const Settings = lazy(() => import("./pages/Settings").then((module) => ({ default: module.Settings })));
const Quality = lazy(() => import("./pages/Quality").then((module) => ({ default: module.Quality })));

export default function App() {
  useClipEvents();
  useUpdateCheck();
  const navigate = useNavigate();
  const setBootstrap = useAppStore((state) => state.setBootstrap);
  const capturePreferences = useAppStore((state) => state.capturePreferences);
  const setCapturePreferences = useAppStore((state) => state.setCapturePreferences);
  useEffect(() => {
    let mounted = true;
    const refresh = () => invoke<BootstrapState>("get_bootstrap_state").then((next) => { if (mounted) setBootstrap(next); }).catch(() => { if (mounted) setBootstrap(null); });
    void refresh();
    return () => { mounted = false; };
  }, [setBootstrap]);
  const bootstrap = useAppStore((state) => state.bootstrap);
  useEffect(() => {
    if (!bootstrap || !["deferred", "loading"].includes(bootstrap.embeddingStatus)) return;
    const interval = window.setInterval(() => { void invoke<BootstrapState>("get_bootstrap_state").then(setBootstrap).catch(() => undefined); }, 1500);
    return () => window.clearInterval(interval);
  }, [bootstrap, setBootstrap]);
  useEffect(() => { invoke<CapturePreferences>("get_capture_preferences").then(setCapturePreferences).catch(() => undefined); }, [setCapturePreferences]);
  useEffect(() => {
    if (!capturePreferences) return;
    document.documentElement.dataset.theme = capturePreferences.appearance;
  }, [capturePreferences]);
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        const sidebarSearch = document.querySelector<HTMLInputElement>("[data-sidebar-search]");
        if (sidebarSearch && sidebarSearch.getClientRects().length > 0) {
          sidebarSearch.focus();
          sidebarSearch.select();
          return;
        }
        navigate("/search");
        requestAnimationFrame(() => document.querySelector<HTMLInputElement>("[data-mnemo-search]")?.focus());
      }
      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "m") {
        event.preventDefault();
        void invoke<CapturePreferences>("toggle_capture").then(setCapturePreferences).catch(() => undefined);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [navigate]);
  
  // Tauri sets url to "/popup" for the popup window
  const isPopup = window.location.pathname === '/popup';

  if (isPopup) {
    return <QuickSearchPopup />;
  }

  return (
    <div className="app-frame">
      <Sidebar />
      <main className="app-content">
        <Suspense fallback={<section className="page"><div className="skeleton-stack"><div /><div /><div /></div></section>}>
          <Routes>
            <Route path="/" element={<Memory />} />
            <Route path="/timeline" element={<Timeline />} />
            <Route path="/search" element={<Search />} />
            <Route path="/session/:id" element={<SessionReconstruction />} />
            <Route path="/graph" element={<MemoryGraph />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/clip/:id" element={<ClipDetail />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/quality" element={<Quality />} />
          </Routes>
        </Suspense>
      </main>
    </div>
  );
}
