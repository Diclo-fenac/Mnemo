import { afterEach, describe, expect, it, vi } from "vitest";

const extensionPath = (file: string) => new URL(`./${file}`, import.meta.url).href;

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("Mnemo Context Bridge", () => {
  it("keeps the service worker alive until its loopback request resolves", async () => {
    let listener: ((message: unknown) => Promise<{ ok: boolean }> | undefined) | undefined;
    const fetchMock = vi.fn().mockResolvedValue(new Response("ok", { status: 200 }));

    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal("chrome", {
      runtime: {
        onMessage: {
          addListener(next: typeof listener) {
            listener = next;
          },
        },
      },
    });

    await import(`${extensionPath("background.js")}?worker=${Date.now()}`);
    const response = await listener?.({
      type: "mnemo-copy-context",
      url: "https://example.com/docs",
      title: "Example docs",
      faviconUrl: "https://example.com/favicon.ico",
      selectedText: "clipboard text",
    });

    expect(response).toEqual({ ok: true });
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:17531/context",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          url: "https://example.com/docs",
          title: "Example docs",
          favicon_url: "https://example.com/favicon.ico",
          selected_text: "clipboard text",
        }),
      }),
    );
  });

  it("reports a failed local request without breaking the copy flow", async () => {
    let listener: ((message: unknown) => Promise<{ ok: boolean }> | undefined) | undefined;
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("Mnemo is closed")));
    vi.stubGlobal("browser", {
      runtime: {
        onMessage: {
          addListener(next: typeof listener) {
            listener = next;
          },
        },
      },
    });

    await import(`${extensionPath("background.js")}?firefox=${Date.now()}`);
    await expect(listener?.({ type: "mnemo-copy-context" })).resolves.toEqual({ ok: false });
  });

  it("sends the active page and selection only when a copy event occurs", async () => {
    let copyListener: (() => void) | undefined;
    const sendMessage = vi.fn().mockResolvedValue({ ok: true });

    vi.stubGlobal("chrome", { runtime: { sendMessage } });
    vi.stubGlobal("window", {
      location: { href: "https://example.com/guide" },
      getSelection: () => ({ toString: () => "  selected note  " }),
    });
    vi.stubGlobal("document", {
      title: "Example guide",
      querySelector: () => ({ href: "https://example.com/favicon.ico" }),
      addEventListener(event: string, listener: () => void) {
        if (event === "copy") copyListener = listener;
      },
    });

    await import(`${extensionPath("content.js")}?content=${Date.now()}`);
    copyListener?.();

    expect(sendMessage).toHaveBeenCalledWith({
      type: "mnemo-copy-context",
      url: "https://example.com/guide",
      title: "Example guide",
      faviconUrl: "https://example.com/favicon.ico",
      selectedText: "selected note",
    });
  });
});
