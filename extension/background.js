const CONTEXT_ENDPOINT = "http://127.0.0.1:17531/context";
const runtime = globalThis.browser?.runtime ?? globalThis.chrome?.runtime;

runtime?.onMessage.addListener((message) => {
  if (message?.type !== "mnemo-copy-context") return undefined;

  // Returning the promise keeps an MV3 service worker alive until the local
  // request finishes. A detached fetch can be cancelled when Chrome suspends it.
  return fetch(CONTEXT_ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      url: message.url,
      title: message.title,
      favicon_url: message.faviconUrl,
      selected_text: message.selectedText,
    }),
  })
    .then((response) => ({ ok: response.ok }))
    .catch(() => ({ ok: false }));
});
