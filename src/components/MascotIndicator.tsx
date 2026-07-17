import { useAppStore } from "../store/app";

export function MascotIndicator() {
  const bootstrap = useAppStore((state) => state.bootstrap);
  const stage = bootstrap?.stage || "clippy";
  const details = {
    clippy: ["📎", "Clippy", "capturing memories"],
    bindor: ["🔗", "Bindor", "recognizing patterns"],
    archivor: ["🗄️", "Archivor", "connecting research"],
  } as const;
  const [icon, label, message] = details[stage] || details.clippy;
  return <div className="mascot" aria-label={`Mnemo intelligence stage: ${label}`}>{icon} {label} <span aria-hidden="true">&middot;</span> {bootstrap?.embeddingStatus === "loading" ? "preparing model" : message}</div>;
}
