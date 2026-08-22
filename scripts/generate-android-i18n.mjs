import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const sourceDirectory = path.join(root, "shared", "i18n");
const resourceDirectory = path.join(
  root,
  "apps",
  "android",
  "core",
  "src",
  "androidMain",
  "res",
);
const checkOnly = process.argv.includes("--check");

function resourceName(key) {
  return key.replaceAll(".", "_dot_").replaceAll("-", "_dash_");
}

function escapeXml(value) {
  return value
    // Android's resource parser treats raw control characters differently
    // from XML. Keep shared JSON strings valid while preserving their UI
    // meaning when Android expands the resource.
    .replaceAll("\\", "\\\\")
    .replaceAll("\n", "\\n")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    // The Android resource merger expands XML entities before aapt2 flattens
    // the merged file. Keep the apostrophe escaped in the resulting Android
    // string syntax so merged resources remain valid as well.
    .replaceAll("'", "\\'");
}

function render(values) {
  const names = new Map();
  const lines = ['<?xml version="1.0" encoding="utf-8"?>', "<resources>"];
  for (const [key, value] of Object.entries(values)) {
    const name = resourceName(key);
    const previousKey = names.get(name);
    if (previousKey)
      throw new Error(`resource name collision: ${previousKey} and ${key}`);
    names.set(name, key);
    const substitutions = value.match(/%(?:\d+\$)?[a-zA-Z]/g) ?? [];
    const percentTokens = value.match(/%/g) ?? [];
    const formattingAttribute = substitutions.length > 1 || percentTokens.length > substitutions.length
      ? ' formatted="false"'
      : "";
    lines.push(`    <string name="${name}"${formattingAttribute}>${escapeXml(value)}</string>`);
  }
  lines.push("</resources>", "");
  return lines.join("\n");
}

const languages = [
  ["english_us", "values"],
  ["tr_tr", "values-tr"],
];

for (const [language, directory] of languages) {
  const sourcePath = path.join(sourceDirectory, `${language}.json`);
  const targetDirectory = path.join(resourceDirectory, directory);
  const targetPath = path.join(targetDirectory, "strings.xml");
  const rendered = render(JSON.parse(fs.readFileSync(sourcePath, "utf8")));
  if (checkOnly) {
    if (
      fs.existsSync(targetPath) &&
      fs.readFileSync(targetPath, "utf8") === rendered
    )
      continue;
    throw new Error(
      `generated Android i18n is out of date: ${path.relative(root, targetPath)}`,
    );
  }
  fs.mkdirSync(targetDirectory, { recursive: true });
  fs.writeFileSync(targetPath, rendered);
}

if (!checkOnly) process.stdout.write("Android i18n resources generated\n");
