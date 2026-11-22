import fs from "node:fs";
import path from "node:path";

// The default skin writes one dialect: standard `var(--token)` colors, one
// class per component and `@media (feature: value)` queries. The engine still
// parses the older spellings so third-party skins keep working.
const skinRoot = path.join(process.cwd(), "shared", "skins", "default");
const css = fs
  .readFileSync(path.join(skinRoot, "style.css"), "utf8")
  .replace(/\/\*[\s\S]*?\*\//g, (match) => match.replace(/[^\n]/g, " "));

const engineDerived = ["theme.input.focus", "theme.toggle.knob.on", "theme.settings.description"];
const legacyClasses = ["action-primary", "action-secondary", "button.primary", "button.secondary", "button.selected", "button.danger", "chip.selected"];

const rules = [
  [/\bink\/[a-z0-9]+/, "ink/<level> color; use var(--ink-<level>)"],
  [/[:\s(,]background\*[0-9.]+/, "background*<factor>; use color-mix(in srgb, var(--color-background) N%, transparent)"],
  [/:\s*(accent|accent\.foreground|danger|background|text\.(primary|secondary|muted)|surface\.raised)\s*;/, "named color; use var(--color-*)"],
  [/(^|[:\s(,])(background|color|border-color)\s*:\s*theme\.(surface|border|input\.[a-z.]+)\s*[;,]/m, "theme.* color; use var(--color-*)"],
  [/:\s*pill\s*;/, "bare `pill`; use var(--radius-pill)"],
  [/^@(mobile|tv|medium|expanded|desktop|compact)\b/m, "@<name> media shorthand; use @media (platform|size: ...)"],
  [/@media\s*\((tv|mobile|desktop)\)/, "@media (tv); use @media (platform: tv)"],
  [/@media[^{]*\(window:/, "@media (window: ...); use (size: ...)"],
];

const failures = [];
const lines = css.split("\n");
for (const [pattern, message] of rules) {
  if (!message) continue;
  lines.forEach((line, index) => {
    const probe = engineDerived.reduce((text, allowed) => text.split(allowed).join(""), line);
    if (pattern.test(probe)) failures.push(`style.css:${index + 1}: ${message}: ${line.trim()}`);
  });
}
for (const name of legacyClasses) {
  const selector = new RegExp(`(^|[\\s,>])\\.?${name.replace(".", "\\.")}(?![\\w-])`);
  lines.forEach((line, index) => {
    if (/^[^{};]*[{,]\s*$/.test(line) && selector.test(line))
      failures.push(`style.css:${index + 1}: legacy selector ${name}: ${line.trim()}`);
  });
}

const jsonFiles = [path.join(skinRoot, "includes.json")];
const screens = path.join(skinRoot, "screens");
for (const file of fs.readdirSync(screens)) if (file.endsWith(".json")) jsonFiles.push(path.join(screens, file));
for (const file of jsonFiles) {
  const text = fs.readFileSync(file, "utf8");
  for (const match of text.matchAll(/"class":\s*"([^"]*)"/g))
    for (const token of match[1].split(/\s+/))
      if (legacyClasses.includes(token))
        failures.push(`${path.relative(process.cwd(), file)}: legacy class ${token}`);
}

if (failures.length > 0) throw new Error(`skin dialect check failed:\n${failures.join("\n")}`);
process.stdout.write("skin dialect verified\n");
