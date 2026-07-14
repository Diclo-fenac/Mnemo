import { useEffect } from "react";
import { useClipsStore } from "../store/clips";
import { ClipCard } from "../components/ClipCard";
import mnemoMark from "../assets/mnemo-mark.svg";

export function Timeline() {
  const { clips, loading, fetchClips } = useClipsStore();

  useEffect(() => {
    fetchClips();
  }, [fetchClips]);

  if (loading && clips.length === 0) {
    return (
      <section className="page">
        <p className="eyebrow">Your memory</p>
        <h1 className="page-title">Loading clips…</h1>
      </section>
    );
  }

  if (clips.length === 0) {
    return (
      <section className="page">
        <p className="eyebrow">Your memory</p>
        <h1 className="page-title">What you copied, remembered.</h1>
        <p className="page-copy">
          Mnemo will quietly collect the threads of your research and arrange
          them into sessions you can revisit. Everything stays on your machine.
        </p>
        <div className="stage-card">
          <strong>No clips yet</strong>
          <p>Copy something — text, a URL, a code snippet — and it will appear here instantly.</p>
          <p style={{ fontSize: 13, opacity: 0.7 }}>
            Quick search: <kbd>Ctrl+Shift+V</kbd>
          </p>
        </div>
        <img className="empty-symbol" src={mnemoMark} alt="" />
      </section>
    );
  }

  return (
    <section className="page">
      <p className="eyebrow">Your memory</p>
      <h1 className="page-title">Timeline</h1>
      <p className="page-copy">
        {clips.length} clip{clips.length !== 1 ? "s" : ""} remembered
      </p>
      <div className="clips-list">
        {clips.map((clip) => (
          <ClipCard key={clip.id} clip={clip} />
        ))}
      </div>
    </section>
  );
}
