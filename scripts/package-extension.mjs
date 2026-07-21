import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const extensionRoot = join(root, "extension");
const outputRoot = join(root, "artifacts", "extension");
const sourceFiles = ["background.js", "content.js"];

function readManifest(name) {
  const path = join(extensionRoot, name);
  if (!existsSync(path)) throw new Error(`Missing extension manifest: ${path}`);
  return JSON.parse(readFileSync(path, "utf8"));
}

function verifySource() {
  const chrome = readManifest("manifest.json");
  const firefox = readManifest("manifest.firefox.json");
  for (const file of sourceFiles) {
    if (!existsSync(join(extensionRoot, file))) throw new Error(`Missing extension source: ${file}`);
  }
  if (chrome.manifest_version !== 3 || firefox.manifest_version !== 3) throw new Error("Both manifests must use Manifest V3");
  if (chrome.version !== firefox.version) throw new Error("Chrome and Firefox extension versions must match");
  for (const manifest of [chrome, firefox]) {
    if (!manifest.host_permissions?.includes("http://127.0.0.1:17531/*")) {
      throw new Error("Both manifests must allow only the documented loopback endpoint");
    }
    if (!manifest.content_scripts?.some((script) => script.js?.includes("content.js"))) {
      throw new Error("Both manifests must install the context bridge content script");
    }
  }
  if (chrome.background?.service_worker !== "background.js") {
    throw new Error("Chrome manifest must use the background service worker");
  }
  if (!firefox.background?.scripts?.includes("background.js") || firefox.background?.service_worker) {
    throw new Error("Firefox manifest must use background.scripts, not service_worker");
  }
  return chrome.version;
}

function packageExtension(manifestName, outputName) {
  const staging = mkdtempSync(join(outputRoot, ".staging-"));
  try {
    cpSync(join(extensionRoot, manifestName), join(staging, "manifest.json"));
    for (const file of sourceFiles) cpSync(join(extensionRoot, file), join(staging, file));
    mkdirSync(outputRoot, { recursive: true });
    const output = join(outputRoot, outputName);
    execFileSync("zip", ["-q", "-r", output, "."], { cwd: staging, stdio: "inherit" });
    return output;
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

const version = verifySource();
if (process.argv.includes("--verify")) {
  console.log(`Extension manifests valid: v${version}`);
  process.exit(0);
}

mkdirSync(outputRoot, { recursive: true });
const chrome = packageExtension("manifest.json", `mnemo-context-bridge-chrome-v${version}.zip`);
const firefox = packageExtension("manifest.firefox.json", `mnemo-context-bridge-firefox-v${version}.xpi`);
console.log(`Created ${basename(chrome)}`);
console.log(`Created ${basename(firefox)}`);
