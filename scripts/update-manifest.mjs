import { createHash, createPrivateKey, generateKeyPairSync, sign } from "node:crypto";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { basename } from "node:path";

const [command, ...args] = process.argv.slice(2);

if (command === "keygen") {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const raw = publicKey.export({ format: "der", type: "spki" }).subarray(-32);
  console.log(`FLUXA_UPDATE_PUBLIC_KEY=${raw.toString("base64")}`);
  console.log(`FLUXA_UPDATE_PRIVATE_KEY=${privateKey.export({ format: "der", type: "pkcs8" }).toString("base64")}`);
} else if (command === "build") {
  const option = (name) => {
    const index = args.indexOf(`--${name}`);
    return index === -1 ? undefined : args[index + 1];
  };
  const version = option("version")?.replace(/^v/, "");
  const repo = option("repo");
  const out = option("out") ?? "latest.json";
  const notes = option("notes") ? readFileSync(option("notes"), "utf8") : "";
  const assets = args.flatMap((arg, i) => (args[i - 1] === "--asset" ? [arg] : []));
  const privateKey = process.env.FLUXA_UPDATE_PRIVATE_KEY;
  if (!version || !repo || !assets.length || !privateKey) {
    console.error("usage: update-manifest.mjs build --version v --repo owner/name --asset key=path... [--notes file] [--out file]");
    process.exit(1);
  }

  const entries = {};
  for (const spec of assets) {
    const [key, path] = spec.split("=");
    entries[key] = {
      url: `https://github.com/${repo}/releases/download/v${version}/${basename(path)}`,
      sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
      size: statSync(path).size,
    };
  }

  const body = JSON.stringify({ version, pub_date: new Date().toISOString(), notes, assets: entries }, null, 2);
  const key = createPrivateKey({ key: Buffer.from(privateKey, "base64"), format: "der", type: "pkcs8" });
  writeFileSync(out, body);
  writeFileSync(`${out}.sig`, sign(null, Buffer.from(body), key).toString("base64"));
  console.log(`${out}: ${Object.keys(entries).length} assets`);
} else {
  console.error("usage: update-manifest.mjs <keygen|build>");
  process.exit(1);
}
