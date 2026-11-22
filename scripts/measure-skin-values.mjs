import fs from "node:fs";
import path from "node:path";

const skin = path.resolve(process.argv[2] ?? "shared/skins/default");
const css = fs.readFileSync(path.join(skin, "style.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
const watched = ["gap", "padding", "margin", "border-radius", "font-size"];

const tokens = {};
for (const m of css.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)) tokens[m[1]] = m[2].trim();

function px(value) {
  const t = value.trim();
  let m = t.match(/^(-?[\d.]+)px$/);
  if (m) return Number(m[1]);
  m = t.match(/^(-?[\d.]+)rem$/);
  if (m) return Number(m[1]) * 16;
  m = t.match(/^var\((--[a-z0-9-]+)\)$/);
  if (m && tokens[m[1]]) return px(tokens[m[1]]);
  if (/^0$/.test(t)) return 0;
  return null;
}

const rows = {};
const re = /([^{}@]+)\{([^{}]*)\}/g;
let m;
while ((m = re.exec(css))) {
  const sels = m[1].split(",").length;
  for (const d of m[2].split(";")) {
    const k = d.indexOf(":");
    if (k < 0) continue;
    const prop = d.slice(0, k).trim();
    if (!watched.includes(prop)) continue;
    for (const part of d.slice(k + 1).trim().split(/\s+(?![^(]*\))/)) {
      const key = prop;
      rows[key] ??= new Map();
      const e = rows[key].get(part) ?? { n: 0, px: px(part) };
      e.n += 1;
      rows[key].set(part, e);
    }
  }
}

for (const prop of watched) {
  const list = [...(rows[prop] ?? [])].sort((a, b) => (a[1].px ?? 1e9) - (b[1].px ?? 1e9));
  console.log(`## ${prop}  (${list.length} distinct, ${list.reduce((s, [, e]) => s + e.n, 0)} uses)`);
  for (const [value, e] of list) console.log(`  ${String(e.n).padStart(3)}x  ${value}${e.px === null ? "" : `  (${e.px}px)`}`);
  console.log();
}
