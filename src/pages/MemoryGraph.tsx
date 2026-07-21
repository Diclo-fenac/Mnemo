import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import * as d3 from "d3-force";
import * as d3Selection from "d3-selection";
import * as d3Zoom from "d3-zoom";
import { ArrowLeft, CircleDot, GitBranch, Minus, Plus, RotateCcw, Sparkles } from "lucide-react";
import type { Clip } from "../types";
import { topicColor } from "../lib/presentation";

type GraphLink = { source: string; target: string; similarity: number; edgeType: string; temporalWeight: number };
type GraphData = { nodes: Clip[]; links: GraphLink[]; state: "building" | "edge_free" | "unavailable" | "ready"; unconnectedCount: number };
type Node = Clip & d3.SimulationNodeDatum;
type Link = d3.SimulationLinkDatum<Node> & Pick<GraphLink, "similarity" | "edgeType" | "temporalWeight">;
type Cluster = { id: string; label: string; nodes: Clip[]; links: GraphLink[]; semanticCount: number; temporalCount: number };

export function MemoryGraph() {
  const [data, setData] = useState<GraphData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeClusterId, setActiveClusterId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const graph = await invoke<GraphData>("get_graph_data", { limit: 200 });
        if (!cancelled) { setData(graph); setError(null); }
      } catch (reason) {
        if (!cancelled) setError(reason instanceof Error ? reason.message : "Unable to load memory graph.");
      }
    }
    void load();
    const retry = window.setInterval(() => { if (!cancelled) void load(); }, 5000);
    const unlisten = listen("clip-added", () => { if (!cancelled) void load(); });
    return () => { cancelled = true; window.clearInterval(retry); unlisten.then((fn) => fn()).catch(() => undefined); };
  }, []);

  const clusters = useMemo(() => data ? deriveClusters(data) : [], [data]);
  const activeCluster = clusters.find((cluster) => cluster.id === activeClusterId) ?? null;

  if (error) return <section className="page graph-state"><div className="graph-state-card"><GitBranch size={22} /><h1>Memory graph unavailable</h1><p>{error}</p><button className="quiet-button" onClick={() => window.location.reload()}>Try again</button></div></section>;
  if (!data) return <section className="page graph-state"><div className="graph-state-card"><Sparkles size={22} /><h1>Reading your memory map</h1><p>Mnemo is checking for meaningful relationships between your captures.</p></div></section>;
  if (data.state !== "ready") return <GraphEmptyState data={data} />;
  if (!activeCluster) return <ClusterOverview clusters={clusters} unconnectedCount={data.unconnectedCount} onExplore={setActiveClusterId} />;
  return <ConnectionExplorer cluster={activeCluster} onBack={() => setActiveClusterId(null)} />;
}

function GraphEmptyState({ data }: { data: GraphData }) {
  const building = data.state === "building";
  const unavailable = data.state === "unavailable";
  return <section className="page graph-state"><div className="graph-state-card graph-state-rich"><span className="graph-state-orbit"><Sparkles size={25} /></span><p className="eyebrow">Memory graph</p><h1>{building ? "Building your first connections" : unavailable ? "Local model unavailable" : "Your captures are still independent"}</h1><p>{building ? "Mnemo is embedding your recent clips locally. The graph appears after at least two distinct clips are embedded and one semantic or temporal relation clears the similarity threshold." : unavailable ? "Mnemo could not embed these captures, so it cannot calculate graph relationships yet. Retry local model preparation from Settings." : "Connections appear when copied items are semantically related or part of the same research thread. Unrelated clips remain searchable without being forced into the graph."}</p><div className="graph-state-details"><span><CircleDot size={15} /> {data.nodes.length} captured memories</span><span><GitBranch size={15} /> {data.unconnectedCount} awaiting a connection</span></div><a className="primary-button" href={unavailable ? "/settings" : "/timeline"}>{unavailable ? "Open Settings" : "Browse recent captures"}</a></div></section>;
}

function ClusterOverview({ clusters, unconnectedCount, onExplore }: { clusters: Cluster[]; unconnectedCount: number; onExplore: (id: string) => void }) {
  return <section className="page graph-cluster-page"><header className="graph-overview-header"><div><p className="eyebrow">Memory graph</p><h1 className="page-title">Knowledge clusters</h1><p className="page-copy">Mnemo only maps real relationships, so every group has a reason to exist.</p></div><div className="graph-overview-stat"><strong>{clusters.length}</strong><span>connected themes</span></div></header><div className="cluster-grid">{clusters.map((cluster) => <article className="cluster-card" key={cluster.id} style={{ "--cluster-color": topicColor(cluster.label) } as CSSProperties}><div className="cluster-card-top"><span className="cluster-dot" /><span>{cluster.nodes.length} captures</span></div><h2>{cluster.label}</h2><p>{cluster.semanticCount} semantic link{cluster.semanticCount === 1 ? "" : "s"} · {cluster.temporalCount} temporal link{cluster.temporalCount === 1 ? "" : "s"}</p><div className="cluster-samples">{cluster.nodes.slice(0, 2).map((clip) => <span key={clip.id}>{clip.content.slice(0, 54)}{clip.content.length > 54 ? "…" : ""}</span>)}</div><button className="quiet-button" onClick={() => onExplore(cluster.id)}>Explore connections <GitBranch size={14} /></button></article>)}</div>{unconnectedCount > 0 && <aside className="unconnected-note"><CircleDot size={16} /><div><strong>{unconnectedCount} unconnected capture{unconnectedCount === 1 ? "" : "s"}</strong><p>These remain searchable. Mnemo keeps them out of the map until a useful relationship exists.</p></div></aside>}</section>;
}

function ConnectionExplorer({ cluster, onBack }: { cluster: Cluster; onBack: () => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const zoomRef = useRef<d3Zoom.ZoomBehavior<HTMLCanvasElement, unknown> | null>(null);
  const selectedClipIdRef = useRef<string | null>(null);
  const [selectedClip, setSelectedClip] = useState<Clip | null>(null);
  const [hoveredClip, setHoveredClip] = useState<Clip | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    selectedClipIdRef.current = null;
    setSelectedClip(null);
    setHoveredClip(null);
  }, [cluster]);

  useEffect(() => {
    if (!canvasRef.current || !containerRef.current) return;
    const canvas = canvasRef.current;
    const context = canvas.getContext("2d");
    if (!context) return;
    const ctx: CanvasRenderingContext2D = context;
    let width = containerRef.current.clientWidth;
    let height = containerRef.current.clientHeight;
    canvas.width = width;
    canvas.height = height;
    const nodes: Node[] = cluster.nodes.map((node) => ({ ...node }));
    const links: Link[] = cluster.links.map((link) => ({ ...link }));
    const simulation = d3.forceSimulation<Node>(nodes).force("charge", d3.forceManyBody().strength(-190)).force("link", d3.forceLink<Node, Link>(links).id((node) => node.id).distance(115)).force("center", d3.forceCenter(width / 2, height / 2)).force("collide", d3.forceCollide(26)).on("tick", draw);
    let transform = d3Zoom.zoomIdentity;
    const zoom = d3Zoom.zoom<HTMLCanvasElement, unknown>().scaleExtent([0.4, 3]).on("zoom", (event) => { transform = event.transform; draw(); });
    zoomRef.current = zoom;
    const selection = d3Selection.select(canvas);
    selection.call(zoom).on("click", (event) => { const [x, y] = d3Selection.pointer(event, canvas); const px = transform.invertX(x); const py = transform.invertY(y); const next = nodes.find((node) => distanceSquared(node, px, py) < 324) ?? null; selectedClipIdRef.current = next?.id ?? null; setSelectedClip(next); draw(); }).on("mousemove", (event) => { const [x, y] = d3Selection.pointer(event, canvas); const px = transform.invertX(x); const py = transform.invertY(y); setHoveredClip(nodes.find((node) => distanceSquared(node, px, py) < 400) ?? null); });
    const resizeObserver = new ResizeObserver(() => { width = containerRef.current?.clientWidth ?? width; height = containerRef.current?.clientHeight ?? height; canvas.width = width; canvas.height = height; simulation.force("center", d3.forceCenter(width / 2, height / 2)); simulation.alpha(0.15).restart(); draw(); });
    resizeObserver.observe(containerRef.current);
    function draw() { ctx.save(); ctx.clearRect(0, 0, width, height); ctx.translate(transform.x, transform.y); ctx.scale(transform.k, transform.k); for (const link of links) { const source = link.source as Node; const target = link.target as Node; if (source.x == null || source.y == null || target.x == null || target.y == null) continue; const temporal = link.edgeType === "temporal" || link.edgeType === "semantic_temporal"; ctx.beginPath(); ctx.moveTo(source.x, source.y); ctx.lineTo(target.x, target.y); ctx.strokeStyle = temporal ? "rgba(217, 179, 186, .56)" : "rgba(175, 201, 214, .58)"; ctx.lineWidth = Math.max(1.2, link.similarity * 2.5); ctx.setLineDash(temporal ? [5, 5] : []); ctx.stroke(); } ctx.setLineDash([]); for (const node of nodes) { if (node.x == null || node.y == null) continue; const selected = selectedClipIdRef.current === node.id; ctx.beginPath(); ctx.arc(node.x, node.y, selected ? 14 : 11, 0, Math.PI * 2); ctx.fillStyle = selected ? "#D7D99F" : topicColor(topicOf(node)); ctx.fill(); ctx.strokeStyle = "#161B18"; ctx.lineWidth = 2; ctx.stroke(); } ctx.restore(); }
    return () => { simulation.stop(); resizeObserver.disconnect(); selection.on(".zoom", null).on("click", null).on("mousemove", null); if (zoomRef.current === zoom) zoomRef.current = null; };
  }, [cluster]);

  function zoomBy(factor: number) { if (canvasRef.current && zoomRef.current) d3Selection.select(canvasRef.current).call(zoomRef.current.scaleBy, factor); }
  function resetView() { if (canvasRef.current && zoomRef.current) d3Selection.select(canvasRef.current).call(zoomRef.current.transform, d3Zoom.zoomIdentity); }
  return <section className="graph-explorer" ref={containerRef}><canvas ref={canvasRef} className="graph-canvas" /><div className="graph-panel graph-title"><button className="graph-back" onClick={onBack}><ArrowLeft size={15} /> Clusters</button><strong>{cluster.label}</strong><span>{cluster.nodes.length} captures · semantic and temporal links</span><div className="graph-controls"><button onClick={() => zoomBy(1.2)} aria-label="Zoom in"><Plus size={15} /></button><button onClick={() => zoomBy(.8)} aria-label="Zoom out"><Minus size={15} /></button><button onClick={resetView} aria-label="Reset graph view"><RotateCcw size={15} /></button></div></div><div className="graph-panel graph-legend"><span>Connection key</span><div><i className="semantic-link" /> Semantic similarity</div><div><i className="temporal-link" /> Semantic + temporal context</div></div>{hoveredClip && <div className="graph-tooltip">{hoveredClip.content.slice(0, 90)}{hoveredClip.content.length > 90 ? "…" : ""}</div>}{selectedClip && <aside className="graph-preview"><button aria-label="Close preview" onClick={() => { selectedClipIdRef.current = null; setSelectedClip(null); }}>×</button><p className="eyebrow">Captured memory</p><p>{selectedClip.content}</p><button className="session-link" onClick={() => navigate(`/clip/${selectedClip.id}`)}>Open clip</button></aside>}</section>;
}

function deriveClusters(data: GraphData): Cluster[] {
  const adjacency = new Map<string, Set<string>>();
  for (const link of data.links) { if (!adjacency.has(link.source)) adjacency.set(link.source, new Set()); if (!adjacency.has(link.target)) adjacency.set(link.target, new Set()); adjacency.get(link.source)?.add(link.target); adjacency.get(link.target)?.add(link.source); }
  const byId = new Map(data.nodes.map((node) => [node.id, node]));
  const seen = new Set<string>();
  const clusters: Cluster[] = [];
  for (const id of adjacency.keys()) { if (seen.has(id)) continue; const queue = [id]; const ids: string[] = []; seen.add(id); while (queue.length) { const current = queue.pop()!; ids.push(current); for (const neighbor of adjacency.get(current) ?? []) if (!seen.has(neighbor)) { seen.add(neighbor); queue.push(neighbor); } } const idSet = new Set(ids); const nodes = ids.flatMap((clipId) => byId.get(clipId) ?? []); const links = data.links.filter((link) => idSet.has(link.source) && idSet.has(link.target)); if (!nodes.length) continue; clusters.push({ id: ids.sort().join("-"), label: topicOf(nodes[0]), nodes, links, semanticCount: links.filter((link) => link.edgeType !== "temporal").length, temporalCount: links.filter((link) => link.edgeType === "temporal" || link.edgeType === "semantic_temporal").length }); }
  return clusters.sort((a, b) => b.nodes.length - a.nodes.length);
}

function distanceSquared(node: Node, x: number, y: number) { return (x - (node.x ?? 0)) ** 2 + (y - (node.y ?? 0)) ** 2; }
function topicOf(clip: Clip): string { const raw = clip.aiContext; if (raw) { try { const context = JSON.parse(raw); const tags = context.topic_tags ?? context.topicTags; if (Array.isArray(tags) && tags[0]) return String(tags[0]); } catch { /* use fallback */ } } return clip.language || clip.contentType || "general"; }
