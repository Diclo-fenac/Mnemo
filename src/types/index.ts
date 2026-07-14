export type IntelligenceStage = "clippy" | "bindor" | "archivor";

export type BootstrapState = {
  databaseReady: boolean;
  embeddingStatus: "deferred" | "loading" | "ready" | "unavailable";
  stage: IntelligenceStage;
};

export type Clip = {
  id: string;
  content: string;
  contentType: "text" | "code" | "url";
  imagePath: string | null;
  sourceUrl: string | null;
  pageTitle: string | null;
  appName: string | null;
  windowTitle: string | null;
  language: string | null;
  sessionId: string | null;
  isPinned: boolean;
  copiedAt: number;
  aiContext: string | null;
  createdAt: string;
};

export type ClipAddedPayload = {
  clipId: string;
  contentPreview: string;
  contentType: string;
  appName: string | null;
  copiedAt: number;
};
