const runtime = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;

document.addEventListener("copy", () => {
  if (!runtime) return;
  const selection = window.getSelection()?.toString().trim() ?? "";
  const favicon = document.querySelector('link[rel~="icon"]')?.href ?? null;
  void runtime
    .sendMessage({
      type: "mnemo-copy-context",
      url: window.location.href,
      title: document.title,
      faviconUrl: favicon,
      selectedText: selection.slice(0, 100_000) || null,
    })
    .catch(() => {
      // Mnemo may be closed. Copying must always remain unaffected.
    });
});
