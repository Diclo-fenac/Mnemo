import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as d3 from "d3-force";
import * as d3Zoom from "d3-zoom";
import * as d3Selection from "d3-selection";
import { ClipCard } from "../components/ClipCard";
import type { Clip } from "../types";

type GraphData = {
  nodes: Clip[];
  links: { source: string; target: string; similarity: number }[];
};

type Node = Clip & d3.SimulationNodeDatum;
type Link = d3.SimulationLinkDatum<Node> & { similarity: number };

export function MemoryGraph() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [data, setData] = useState<GraphData | null>(null);
  const [selectedClip, setSelectedClip] = useState<Clip | null>(null);

  useEffect(() => {
    invoke<GraphData>("get_graph_data").then(setData).catch(console.error);
  }, []);

  useEffect(() => {
    if (!data || !canvasRef.current || !containerRef.current) return;
    
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = containerRef.current.clientWidth;
    const height = containerRef.current.clientHeight;

    canvas.width = width;
    canvas.height = height;

    const nodes: Node[] = data.nodes.map(n => ({ ...n }));
    const links: Link[] = data.links.map(l => ({ ...l, source: l.source, target: l.target }));

    const simulation = d3.forceSimulation<Node>(nodes)
      .force("charge", d3.forceManyBody().strength(-150))
      .force("link", d3.forceLink<Node, Link>(links).id(d => d.id).distance(100))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .on("tick", ticked);

    let transform = d3Zoom.zoomIdentity;

    d3Selection.select(canvas)
      .call(d3Zoom.zoom<HTMLCanvasElement, unknown>()
        .scaleExtent([0.1, 4])
        .on("zoom", (e) => {
          transform = e.transform;
          ticked();
        })
      )
      .on("click", (e) => {
        const [x, y] = d3Selection.pointer(e, canvas);
        const simX = transform.invertX(x);
        const simY = transform.invertY(y);
        
        let clickedNode = null;
        for (const node of nodes) {
          const dx = simX - (node.x || 0);
          const dy = simY - (node.y || 0);
          if (dx * dx + dy * dy < 200) {
            clickedNode = node;
            break;
          }
        }
        
        setSelectedClip(clickedNode);
      });

    function ticked() {
      if (!ctx || !canvas) return;
      ctx.save();
      ctx.clearRect(0, 0, width, height);
      ctx.translate(transform.x, transform.y);
      ctx.scale(transform.k, transform.k);

      ctx.beginPath();
      for (const link of links) {
        const source = link.source as Node;
        const target = link.target as Node;
        if (source.x != null && source.y != null && target.x != null && target.y != null) {
          ctx.moveTo(source.x, source.y);
          ctx.lineTo(target.x, target.y);
        }
      }
      ctx.strokeStyle = "rgba(180, 175, 165, 0.4)";
      ctx.lineWidth = 1.5;
      ctx.stroke();

      for (const node of nodes) {
        if (node.x != null && node.y != null) {
          ctx.beginPath();
          ctx.arc(node.x, node.y, 10, 0, 2 * Math.PI);
          ctx.fillStyle = selectedClip?.id === node.id ? "#F8C557" : "#5B2F06";
          ctx.fill();
          ctx.strokeStyle = "#F5F1E8";
          ctx.lineWidth = 2;
          ctx.stroke();
        }
      }
      ctx.restore();
    }

    return () => {
      simulation.stop();
    };
  }, [data, selectedClip]);

  return (
    <section className="page flex h-full p-0 overflow-hidden relative" ref={containerRef}>
      <canvas 
        ref={canvasRef} 
        className="flex-1 cursor-grab active:cursor-grabbing outline-none"
      />
      {selectedClip && (
        <div className="absolute right-0 top-0 bottom-0 w-80 bg-[var(--color-soft-white)] border-l border-[var(--color-soft-border)] p-6 shadow-xl overflow-y-auto">
          <div className="flex justify-between items-center mb-6">
            <h3 className="font-medium text-[var(--color-charcoal)]">Preview</h3>
            <button 
              onClick={() => setSelectedClip(null)}
              className="text-[var(--color-muted)] hover:text-[var(--color-charcoal)] transition-colors"
            >
              &times;
            </button>
          </div>
          <ClipCard clip={selectedClip} />
        </div>
      )}
    </section>
  );
}
