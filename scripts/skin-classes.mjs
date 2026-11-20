import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const skinRoot = path.join(root, "shared", "skins", "default");
export const contractPath = path.join(root, "shared", "contracts", "skin-classes.json");

const helperPattern =
  /^(row|column|stack|spacer|text|input)(-(gap|items|justify|grow|w|h|pad)[a-z0-9-]*)+$/;

function walk(node, visit) {
  if (Array.isArray(node)) {
    for (const item of node) walk(item, visit);
    return;
  }
  if (node && typeof node === "object") {
    visit(node);
    for (const value of Object.values(node)) walk(value, visit);
  }
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function cssSelectorClasses(source) {
  const text = source.replace(/\/\*[\s\S]*?\*\//g, "");
  const found = new Set();
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === "{" || ch === "}" || ch === ";") {
      if (ch === "{") {
        const prelude = text.slice(start, i);
        if (!prelude.trim().startsWith("@")) {
          for (const match of prelude.matchAll(/\.([A-Za-z_][\w-]*)/g))
            found.add(match[1]);
        }
      }
      start = i + 1;
    }
  }
  return found;
}

export function collectClasses() {
  const classes = new Map();
  const add = (name, element) => {
    if (!classes.has(name)) classes.set(name, new Set());
    if (element) classes.get(name).add(element);
  };
  const files = [path.join(skinRoot, "includes.json")];
  const screens = path.join(skinRoot, "screens");
  for (const name of fs.readdirSync(screens).sort())
    if (name.endsWith(".json")) files.push(path.join(screens, name));
  for (const file of files) {
    walk(readJson(file), (node) => {
      if (typeof node.class !== "string") return;
      const element = typeof node.type === "string" ? node.type : "include";
      for (const token of node.class.split(/\s+/)) {
        if (token && !token.includes("{")) add(token, element);
      }
    });
  }
  for (const name of cssSelectorClasses(
    fs.readFileSync(path.join(skinRoot, "style.css"), "utf8"),
  ))
    add(name, null);
  return classes;
}

export function isLayoutHelper(name) {
  return helperPattern.test(name);
}

export function generate() {
  const existing = fs.existsSync(contractPath) ? readJson(contractPath) : null;
  const previous = new Map(
    (existing?.classes ?? []).map((entry) => [entry.name, entry]),
  );
  const found = collectClasses();
  const entries = [];
  for (const [name, elements] of found) {
    const old = previous.get(name);
    const element = [...elements].sort().join(" ") || (old?.element ?? "");
    entries.push({
      name,
      element,
      meaning: old?.meaning ?? "",
      stability: old?.stability ?? "internal",
      since: old?.since ?? existing?.version ?? 1,
    });
  }
  entries.sort((a, b) => a.name.localeCompare(b.name));
  const missingStable = [...previous.values()].filter(
    (entry) => entry.stability === "stable" && !found.has(entry.name),
  );
  return {
    contract: {
      version: existing?.version ?? 1,
      classes: entries,
      removed: existing?.removed ?? [],
    },
    missingStable,
    added: entries.filter((entry) => !previous.has(entry.name)).map((e) => e.name),
    dropped: [...previous.keys()].filter((name) => !found.has(name)),
  };
}

if (process.argv[1] === new URL(import.meta.url).pathname) {
  const { contract, missingStable, added, dropped } = generate();
  fs.writeFileSync(contractPath, JSON.stringify(contract, null, 2) + "\n");
  process.stdout.write(
    `skin-classes.json: ${contract.classes.length} classes, ${added.length} new (internal), ${dropped.length} no longer used\n`,
  );
  for (const entry of missingStable)
    process.stdout.write(
      `stable class no longer used: ${entry.name} (move it to "removed" and bump "version")\n`,
    );
}
