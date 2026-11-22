import fs from "node:fs";
import path from "node:path";

const skin = path.resolve(process.argv[2] ?? "shared/skins/default");
const top = Number(process.argv[3] ?? 25);
const css = fs.readFileSync(path.join(skin, "style.css"), "utf8");

const layoutProps = new Set([
  "gap", "padding", "align-items", "justify-content", "flex-grow", "flex-fill", "flex-direction",
  "width", "height", "max-width", "max-height", "min-width", "min-height", "aspect-ratio",
  "column-width", "columns", "display", "margin",
]);

function parse(source) {
  const text = source.replace(/\/\*[\s\S]*?\*\//g, "");
  const rules = [];
  let i = 0;
  function block(media) {
    while (i < text.length) {
      while (/\s/.test(text[i] ?? "")) i++;
      if (i >= text.length || text[i] === "}") return;
      const start = i;
      while (i < text.length && text[i] !== "{" && text[i] !== ";") i++;
      const head = text.slice(start, i).trim();
      if (text[i] === ";") { i++; continue; }
      i++;
      if (head.startsWith("@media")) {
        block(head.slice(6).trim());
        i++;
      } else if (head.startsWith("@")) {
        let depth = 1;
        while (i < text.length && depth) {
          if (text[i] === "{") depth++;
          if (text[i] === "}") depth--;
          i++;
        }
      } else {
        const bodyStart = i;
        while (i < text.length && text[i] !== "}") i++;
        const body = text.slice(bodyStart, i);
        i++;
        const decls = [];
        for (const part of body.split(";")) {
          const k = part.indexOf(":");
          if (k < 0) continue;
          decls.push([part.slice(0, k).trim(), part.slice(k + 1).trim().replace(/\s+/g, " ")]);
        }
        rules.push({
          sels: head.split(",").map((s) => s.trim().replace(/\s+/g, " ")).filter(Boolean),
          decls,
          media,
        });
      }
    }
  }
  block(null);
  return rules;
}

const rules = parse(css);
const bytes = (s) => Buffer.byteLength(s);

function resolveScreens() {
  const dir = path.join(skin, "screens");
  const includes = JSON.parse(fs.readFileSync(path.join(skin, "includes.json"), "utf8"));
  const collect = (node, classes, tags, incs) => {
    if (Array.isArray(node)) return node.forEach((n) => collect(n, classes, tags, incs));
    if (!node || typeof node !== "object") return;
    if (typeof node.class === "string") for (const c of node.class.split(/\s+/)) classes.add(c);
    if (typeof node.tag === "string") tags.add(node.tag);
    if (typeof node.include === "string") incs.add(node.include);
    for (const v of Object.values(node)) collect(v, classes, tags, incs);
  };
  const own = (node) => {
    const c = new Set(), t = new Set(), n = new Set();
    collect(node, c, t, n);
    return { classes: c, tags: t, incs: n };
  };
  const incInfo = Object.fromEntries(Object.entries(includes).map(([k, v]) => [k, own(v)]));
  const screens = {};
  for (const f of fs.readdirSync(dir)) {
    if (!f.endsWith(".json")) continue;
    const info = own(JSON.parse(fs.readFileSync(path.join(dir, f), "utf8")));
    const seen = new Set();
    const queue = [...info.incs];
    while (queue.length) {
      const name = queue.pop();
      if (seen.has(name) || !incInfo[name]) continue;
      seen.add(name);
      for (const c of incInfo[name].classes) info.classes.add(c);
      for (const t of incInfo[name].tags) info.tags.add(t);
      queue.push(...incInfo[name].incs);
    }
    screens[f.replace(".json", "")] = info;
  }
  return screens;
}

const screens = resolveScreens();

function screensFor(selector) {
  const parts = selector.match(/\.[A-Za-z0-9_-]+/g)?.map((c) => c.slice(1)) ?? [];
  if (!parts.length) return [];
  return Object.entries(screens)
    .filter(([, info]) => parts.every((c) => info.classes.has(c)))
    .map(([name]) => name);
}

const uniqueSelectors = new Set(rules.flatMap((r) => r.sels));
const lines = css.split("\n").length;
console.log(`# style.css: ${lines} lines, ${bytes(css)} bytes, ${rules.length} rules, ${uniqueSelectors.size} unique selectors\n`);

const declKey = (decls) => decls.map(([k, v]) => `${k}:${v}`).sort().join("; ");

function cluster(title, keyOf, filter) {
  const map = new Map();
  for (const r of rules) {
    const key = keyOf(r);
    if (!key) continue;
    if (!map.has(key)) map.set(key, []);
    map.get(key).push(r);
  }
  const rows = [...map.entries()]
    .map(([key, rs]) => {
      const sels = [...new Set(rs.flatMap((r) => r.sels))];
      const scr = new Set(sels.flatMap(screensFor));
      return { key, rules: rs.length, sels, screens: scr.size, waste: (rs.length - 1) * bytes(key) };
    })
    .filter((r) => r.sels.length >= 2 && (!filter || filter(r)))
    .sort((a, b) => b.waste - a.waste || b.sels.length - a.sels.length);
  console.log(`## ${title}  (${rows.length} clusters, ~${rows.reduce((s, r) => s + r.waste, 0)} bytes redundant)\n`);
  for (const r of rows.slice(0, top)) {
    console.log(`- x${r.sels.length} sels, ${r.screens} screens, ~${r.waste}B  {${r.key}}`);
    console.log(`    ${r.sels.slice(0, 8).join(" | ")}${r.sels.length > 8 ? ` | +${r.sels.length - 8}` : ""}`);
  }
  console.log();
}

cluster("Identical declaration sets (whole rule)", (r) => (r.decls.length ? declKey(r.decls) : null));

const lookKey = (r) => {
  const d = r.decls.filter(([k]) => !layoutProps.has(k));
  return d.length >= 2 ? declKey(d) : null;
};
cluster("Identical look (non-layout) declarations, 2+ props", lookKey);

const layoutKey = (r) => {
  const d = r.decls.filter(([k]) => layoutProps.has(k));
  return d.length >= 2 ? declKey(d) : null;
};
cluster("Identical layout declarations, 2+ props", layoutKey);

console.log(`## Declaration frequency (property: value, count of rules)\n`);
const freq = new Map();
for (const r of rules) for (const d of r.decls) {
  const k = `${d[0]}: ${d[1]}`;
  freq.set(k, (freq.get(k) ?? 0) + 1);
}
for (const [k, n] of [...freq.entries()].sort((a, b) => b[1] - a[1]).slice(0, top)) console.log(`- ${n}x  ${k}`);
console.log();

console.log(`## Descendant rules grouped by owning root class\n`);
const roots = new Map();
for (const r of rules) for (const s of r.sels) {
  const compounds = s.split(" ");
  if (compounds.length < 2) continue;
  const root = compounds[0].replace(/:[a-z-]+/g, "");
  roots.set(root, (roots.get(root) ?? 0) + 1);
}
const rootRows = [...roots.entries()].sort((a, b) => b[1] - a[1]);
console.log(`${rootRows.length} roots, ${rootRows.reduce((s, r) => s + r[1], 0)} descendant selectors`);
for (const [root, n] of rootRows.slice(0, top)) console.log(`- ${n}  ${root}`);
console.log();

console.log(`## Per-element property signatures (how many distinct selectors set a given look combo)\n`);
const combos = new Map();
for (const r of rules) {
  const props = [...new Set(r.decls.map(([k]) => k))].filter((k) => !layoutProps.has(k)).sort().join(",");
  if (!props) continue;
  combos.set(props, (combos.get(props) ?? 0) + r.sels.length);
}
for (const [k, n] of [...combos.entries()].sort((a, b) => b[1] - a[1]).slice(0, 12)) console.log(`- ${n} selectors  [${k}]`);
console.log();

console.log(`## Screens with the most own selectors\n`);
const perScreen = {};
for (const sel of uniqueSelectors) for (const s of screensFor(sel)) perScreen[s] = (perScreen[s] ?? 0) + 1;
for (const [s, n] of Object.entries(perScreen).sort((a, b) => b[1] - a[1])) console.log(`- ${s}: ${n}`);
