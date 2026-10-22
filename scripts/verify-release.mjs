import { createHash, createPublicKey, verify } from "node:crypto";
import { readFileSync } from "node:fs";

const args = process.argv.slice(2);
const option = (name) => args[args.indexOf(`--${name}`) + 1];
const repo = option("repo");
const tag = option("tag");
if (!repo || !tag) {
  console.error("usage: verify-release.mjs --repo owner/name --tag vX.Y.Z");
  process.exit(1);
}

const base = `https://github.com/${repo}/releases/download/${tag}`;
const get = async (url) => {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
};

const spkiPrefix = Buffer.from("302a300506032b6570032100", "hex");
const keys = readFileSync("shared/update-public-key.txt", "utf8").split(/\s+/).filter(Boolean);
const manifest = await get(`${base}/latest.json`);
const signature = Buffer.from((await get(`${base}/latest.json.sig`)).toString().trim(), "base64");
const signed = keys.some((key) =>
  verify(null, manifest, createPublicKey({ key: Buffer.concat([spkiPrefix, Buffer.from(key, "base64")]), format: "der", type: "spki" }), signature),
);
if (!signed) throw new Error("latest.json signature does not match the compiled-in public key");

const { version, assets } = JSON.parse(manifest);
if (version !== tag.replace(/^v/, "")) throw new Error(`manifest version ${version} does not match ${tag}`);

const required = readFileSync("shared/update-required-assets.txt", "utf8").split("\n").filter(Boolean);
const missing = required.filter((key) => !(key in assets));
if (missing.length) throw new Error(`manifest is missing ${missing.join(", ")}`);

for (const [key, asset] of Object.entries(assets)) {
  const body = await get(asset.url);
  const sha256 = createHash("sha256").update(body).digest("hex");
  if (body.length !== asset.size || sha256 !== asset.sha256) throw new Error(`${key}: ${asset.url} does not match the manifest`);
  console.log(`ok ${key}`);
}
