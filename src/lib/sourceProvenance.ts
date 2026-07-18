import type { Clip } from "../types";

export type SourceProvenance =
  | { kind: "verified_browser"; label: string; detail: string }
  | { kind: "detected_app"; label: string; detail: string }
  | { kind: "unavailable"; label: "Source unavailable"; detail: string };

export function deriveSourceProvenance(clip: Pick<Clip, "sourceUrl" | "pageTitle" | "appName" | "windowTitle">): SourceProvenance {
  const sourceUrl = clip.sourceUrl?.trim();
  if (sourceUrl) {
    let fallbackLabel = "Browser page";
    try {
      fallbackLabel = new URL(sourceUrl).hostname || fallbackLabel;
    } catch {
      // Keep the page title or neutral label when an extension payload is malformed.
    }
    return {
      kind: "verified_browser",
      label: clip.pageTitle?.trim() || fallbackLabel,
      detail: sourceUrl,
    };
  }

  const appName = clip.appName?.trim();
  if (appName && appName.toLowerCase() !== "unknown") {
    return {
      kind: "detected_app",
      label: appName,
      detail: clip.windowTitle?.trim() || "Detected application",
    };
  }

  return {
    kind: "unavailable",
    label: "Source unavailable",
    detail: "Mnemo could not identify where this clip came from.",
  };
}
