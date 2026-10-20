import { readFileSync, statSync, writeFileSync } from "node:fs";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    version: { type: "string" },
    repo: { type: "string" },
    ipa: { type: "string" },
    "bundle-id": { type: "string" },
    build: { type: "string" },
    "min-os": { type: "string" },
    notes: { type: "string" },
    out: { type: "string", default: "sidestore.json" },
  },
});

const { version, repo, ipa, "bundle-id": bundleId, build, "min-os": minOs, notes, out } = values;
if (!version || !repo || !ipa || !bundleId || !build || !minOs) {
  console.error("usage: sidestore-source.mjs --version v --repo owner/name --ipa file --bundle-id id --build n --min-os x.y [--notes file] [--out file]");
  process.exit(1);
}

const tag = version.startsWith("v") ? version : `v${version}`;
const number = tag.slice(1);
const base = `https://github.com/${repo}/releases`;
const description = notes ? readFileSync(notes, "utf8").trim().slice(0, 1500) : "";
const entry = {
  version: number,
  buildVersion: build,
  date: new Date().toISOString(),
  downloadURL: `${base}/download/${tag}/${ipa.split("/").pop()}`,
  size: statSync(ipa).size,
  localizedDescription: description,
  minOSVersion: minOs,
};

const source = {
  name: "Fluxa",
  identifier: `io.github.${repo.split("/").join(".").toLowerCase()}`,
  subtitle: "Your media, in one place.",
  description: "Fluxa builds for SideStore.",
  iconURL: `https://raw.githubusercontent.com/${repo}/${tag}/packaging/icons/fluxa.png`,
  website: `https://github.com/${repo}`,
  sourceURL: `https://raw.githubusercontent.com/${repo}/HEAD/sidestore.json`,
  apps: [
    {
      name: "Fluxa",
      bundleIdentifier: bundleId,
      developerName: "Fluxa",
      subtitle: "Your media, in one place.",
      localizedDescription: "Your media, in one place.",
      iconURL: `https://raw.githubusercontent.com/${repo}/${tag}/packaging/icons/fluxa.png`,
      tintColor: "#FFFFFF",
      screenshotURLs: [],
      version: entry.version,
      versionDate: entry.date,
      versionDescription: entry.localizedDescription,
      downloadURL: entry.downloadURL,
      size: entry.size,
      versions: [entry],
      appPermissions: { entitlements: [], privacy: {} },
    },
  ],
  news: [],
};

writeFileSync(out, JSON.stringify(source, null, 2) + "\n");
console.log(`${out}: ${bundleId} ${number}`);
