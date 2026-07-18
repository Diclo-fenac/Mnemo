import { describe, expect, it } from "vitest";
import { deriveSourceProvenance } from "./sourceProvenance";

describe("deriveSourceProvenance", () => {
  it("treats browser URL and title as verified context", () => {
    expect(deriveSourceProvenance({ sourceUrl: "https://docs.rs/", pageTitle: "docs.rs", appName: "Chrome", windowTitle: "" })).toEqual({
      kind: "verified_browser",
      label: "docs.rs",
      detail: "https://docs.rs/",
    });
  });

  it("uses a detected application when browser context is absent", () => {
    expect(deriveSourceProvenance({ sourceUrl: null, pageTitle: null, appName: "Code", windowTitle: "lib.rs" })).toEqual({
      kind: "detected_app",
      label: "Code",
      detail: "lib.rs",
    });
  });

  it("does not turn unknown metadata into a source guess", () => {
    expect(deriveSourceProvenance({ sourceUrl: null, pageTitle: null, appName: "Unknown", windowTitle: "" }).kind).toBe("unavailable");
  });

  it("does not fail when a malformed URL is supplied", () => {
    expect(deriveSourceProvenance({ sourceUrl: "not-a-url", pageTitle: "Copied page", appName: null, windowTitle: null }).label).toBe("Copied page");
  });
});
