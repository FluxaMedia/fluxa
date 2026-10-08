import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const version = process.argv[2]?.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version ?? "")) {
  console.error("usage: set-version.mjs <x.y.z>");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const manifests = [
  join(root, "core/fluxa-core/Cargo.toml"),
  ...readdirSync(join(root, "native"))
    .map((name) => join(root, "native", name, "Cargo.toml"))
    .filter(existsSync),
];

for (const path of manifests) {
  const text = readFileSync(path, "utf8");
  const next = text.replace(/(\[package\][^\[]*?\nversion\s*=\s*)"[^"]*"/, `$1"${version}"`);
  if (next !== text) writeFileSync(path, next);
}
console.log(`stamped ${version} into ${manifests.length} manifests`);
