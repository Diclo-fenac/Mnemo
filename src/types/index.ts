export type IntelligenceStage = "clippy" | "bindor" | "archivor";

export type BootstrapState = {
  databaseReady: boolean;
  onboardingCompleted: boolean;
  embeddingStatus: "deferred" | "loading" | "ready" | "unavailable";
  stage: IntelligenceStage;
};

export type UpdateState = {
  status: "idle" | "checking" | "current" | "available" | "error";
  version?: string;
  notes?: string;
};

export type CapturePreferences = {
  captureEnabled: boolean;
  browserContextEnabled: boolean;
  autoDeleteDays: number | null;
  appearance: "dark" | "light" | "system";
  onboardingCompleted: boolean;
};

export type AiSettings = {
  provider: "none" | "ollama" | "openai" | "gemini";
  model: string;
  ollamaUrl: string;
  hasApiKey: boolean;
  cloudConsent: boolean;
};

export type EmbeddingModelStatus = {
  modelId: string;
  cached: boolean;
  runtimeStatus: "deferred" | "loading" | "ready" | "unavailable";
  error: string | null;
  cachedModels: string[];
  pendingModel: string | null;
  pendingState: string | null;
};

export type GroundedAnswer = {
  answer: string;
  citations: string[];
  confidence: "high" | "medium" | "low";
  source: string;
  fallbackReason: string | null;
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

export type MatchReason = {
  reasonType: string;
  label: string;
  weight: number;
};

export type SearchResult = {
  clip: Clip;
  duplicateCount: number;
  searchType: string;
  score: number;
  matchReasons: MatchReason[];
};

export type SessionSummary = {
  id: string;
  label: string;
  summary: string;
  keyTopics: string[];
  sourceApps: string[];
  sourceUrls: string[];
  clipCount: number;
  startedAt: number;
  endedAt: number;
  durationMs: number;
};

export type SourceStat = { label: string; count: number; sourceType: "web" | "app" };
export type SessionConnection = { clipId: string; contentPreview: string; similarity: number; copiedAt: number };
export type SessionReconstruction = { session: SessionSummary; clips: Clip[]; sourceBreakdown: SourceStat[]; connections: SessionConnection[] };
export type RelatedClip = { id: string; content: string; sourceUrl: string | null; pageTitle: string | null; appName: string | null; copiedAt: number; similarity: number; edgeType: string };
export type ClipContext = { source: string; likelyPurpose: string; topicTags: string[] };
