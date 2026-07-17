import traceLoop from "../assets/trace-loop.svg";
import traceLoopMono from "../assets/trace-loop-mono.svg";

type BrandMarkProps = {
  size?: number;
  variant?: "full" | "mono" | "wordmark";
  className?: string;
};

export function BrandMark({ size = 28, variant = "full", className }: BrandMarkProps) {
  const source = variant === "mono" ? traceLoopMono : traceLoop;
  return (
    <span className={`brand-mark brand-mark-${variant} ${className ?? ""}`.trim()}>
      <img src={source} width={size} height={size} alt="" aria-hidden="true" />
      {variant === "wordmark" && <span className="brand-wordmark">Mnemo</span>}
    </span>
  );
}
