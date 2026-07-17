import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type QualityMetrics = {
  totalClips: number;
  embeddedClips: number;
  pendingClips: number;
  failedClips: number;
  skippedClips: number;
  embeddingCoverage: number;
  duplicateCount: number;
  edgeCount: number;
  emptySearchRate: number;
  ctrByPosition: { band: string; impressions: number; clicks: number; ctr: number }[];
};

export function Quality() {
  const [metrics, setMetrics] = useState<QualityMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<QualityMetrics>("get_quality_metrics").then(setMetrics).catch((reason) => setError(reason instanceof Error ? reason.message : "Unable to load local quality metrics."));
  }, []);

  if (!metrics) return <section className="page"><h1 className="page-title">Quality</h1><p className="page-copy" role={error ? "alert" : undefined}>{error || "Loading local engine metrics…"}</p></section>;

  return (
    <section className="page max-w-4xl mx-auto">
      <p className="eyebrow">Local engine</p>
      <h1 className="page-title">Quality dashboard</h1>
      <p className="page-copy">Search and memory quality signals stay on this device.</p>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mt-8">
        <Metric label="Embedding coverage" value={`${(metrics.embeddingCoverage * 100).toFixed(0)}%`} />
        <Metric label="Duplicates" value={metrics.duplicateCount} />
        <Metric label="Memory edges" value={metrics.edgeCount} />
        <Metric label="Empty searches" value={`${(metrics.emptySearchRate * 100).toFixed(1)}%`} />
      </div>
      <div className="stage-card mt-6">
        <strong>Embedding queue</strong>
        <p>{metrics.embeddedClips} embedded · {metrics.pendingClips} pending · {metrics.failedClips} failed · {metrics.skippedClips} skipped</p>
      </div>
      <div className="stage-card mt-6">
        <strong>Click-through by result position</strong>
        <div className="mt-4 space-y-3">
          {metrics.ctrByPosition.map((band) => (
            <div key={band.band} className="flex items-center gap-3 text-sm">
              <span className="w-12">{band.band}</span>
              <div className="h-2 flex-1 rounded-full bg-[var(--color-warm-sand)] overflow-hidden">
                <div className="h-full bg-[var(--color-amber)]" style={{ width: `${Math.min(100, band.ctr * 100)}%` }} />
              </div>
              <span className="w-16 text-right">{(band.ctr * 100).toFixed(1)}%</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: string | number }) {
  return <div className="stage-card"><p className="eyebrow">{label}</p><strong className="text-2xl">{value}</strong></div>;
}
