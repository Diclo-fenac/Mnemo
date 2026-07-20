import { readFileSync } from "node:fs";

const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const configOnly = process.argv.includes("--config-only");
const updater = config.plugins?.updater;
if (!updater?.pubkey || updater.pubkey.includes("replace") || updater.pubkey.length < 40) {
  throw new Error("Updater public key is missing or looks like a placeholder");
}
if (!Array.isArray(updater.endpoints) || updater.endpoints.length === 0 || !updater.endpoints.every((endpoint) => endpoint.startsWith("https://"))) {
  throw new Error("Updater endpoint must use HTTPS");
}
if (configOnly) {
  console.log("Updater configuration verified (signing secret check skipped)");
  process.exit(0);
}
if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
  throw new Error("TAURI_SIGNING_PRIVATE_KEY secret is missing");
}
if (process.env.TAURI_SIGNING_PRIVATE_KEY.includes("BEGIN") || process.env.TAURI_SIGNING_PRIVATE_KEY.includes("replace")) {
  throw new Error("TAURI_SIGNING_PRIVATE_KEY must be a configured Tauri signing key, not a placeholder");
}
console.log(`Updater configuration verified for ${updater.endpoints.length} HTTPS endpoint(s)`);
