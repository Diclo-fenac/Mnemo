import { Route, Routes } from "react-router-dom";
import { MascotIndicator } from "./components/MascotIndicator";
import { Sidebar } from "./components/Sidebar";
import { ClipDetail } from "./pages/ClipDetail";
import { MemoryGraph } from "./pages/MemoryGraph";
import { Search } from "./pages/Search";
import { SessionReconstruction } from "./pages/SessionReconstruction";
import { Settings } from "./pages/Settings";
import { Timeline } from "./pages/Timeline";
import { useClipEvents } from "./hooks/useClipEvents";

export default function App() {
  useClipEvents();

  return (
    <div className="app-frame">
      <Sidebar />
      <main className="app-content">
        <Routes>
          <Route path="/" element={<Timeline />} />
          <Route path="/search" element={<Search />} />
          <Route path="/session/:id" element={<SessionReconstruction />} />
          <Route path="/graph" element={<MemoryGraph />} />
          <Route path="/clip/:id" element={<ClipDetail />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
      <MascotIndicator />
    </div>
  );
}
