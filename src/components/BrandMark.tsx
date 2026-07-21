import mnemoLogo from "../assets/mnemo-logo.png";

type BrandMarkProps = {
  size?: number;
  variant?: "full" | "mono" | "wordmark";
  className?: string;
};

export function BrandMark({ size = 28, variant = "full", className }: BrandMarkProps) {
  const wordmark = variant === "wordmark";
  const width = wordmark ? size * 5.2 : size;
  return (
    <span
      className={`brand-mark brand-mark-${variant} ${className ?? ""}`.trim()}
      style={{ width, height: size }}
      aria-label="Mnemo"
    >
      <img
        className="brand-mark-logo"
        src={mnemoLogo}
        width={wordmark ? width : size * 2.2}
        height={wordmark ? size * 2.2 : size * 1.1}
        alt="Mnemo"
      />
    </span>
  );
}
