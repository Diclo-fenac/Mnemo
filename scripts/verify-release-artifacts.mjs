import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const [metadataDir = "release-assets", artifactDir = "platform-artifacts"] = process.argv.slice(2);

function filesUnder(root) {
  if (!existsSync(root)) return [];
  const result = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) result.push(...filesUnder(path));
    else if (statSync(path).size > 0) result.push(path);
  }
  return result;
}

const metadata = join(metadataDir, "latest.json");
if (!existsSync(metadata)) throw new Error(`Missing updater metadata: ${metadata}`);
const latest = JSON.parse(readFileSync(metadata, "utf8"));
if (!latest.version || !latest.platforms || typeof latest.platforms !== "object" || Object.keys(latest.platforms).length === 0) {
  throw new Error("Updater metadata has no version or platform entries");
}

const signatures = filesUnder(metadataDir).filter((path) => path.endsWith(".sig"));
if (signatures.length === 0) throw new Error("Missing updater signature");

const artifacts = filesUnder(artifactDir);
const hasLinux = artifacts.some((path) => /AppImage$/.test(path));
const hasMac = artifacts.some((path) => /\.dmg$/.test(path));
const hasWindows = artifacts.some((path) => /\.msi$|\.exe$/.test(path));
if (!hasLinux || !hasMac || !hasWindows) {
  throw new Error(`Missing platform artifacts: linux=${hasLinux} macos=${hasMac} windows=${hasWindows}`);
}

console.log(`Release metadata v${latest.version} and ${signatures.length} signature(s) verified`);
console.log(`Platform artifacts verified: ${artifacts.map((path) => relative(artifactDir, path)).join(", ")}`);
