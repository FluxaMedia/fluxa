import fs from "node:fs";
import path from "node:path";

const file = path.join(process.cwd(), "shared", "skins", "default", "style.css");
const css = fs.readFileSync(file, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");

const rules = [];
for (const m of css.matchAll(/([^{}@]+)\{([^{}]*)\}/g)) {
  const head = m[1].trim();
  if (head.startsWith(":root")) continue;
  for (const d of m[2].split(";")) {
    const k = d.indexOf(":");
    if (k < 0) continue;
    rules.push({ head, prop: d.slice(0, k).trim(), value: d.slice(k + 1).trim() });
  }
}

const space = /^(0|var\(--space-[1-9]\))$/;
const checks = {
  gap: (v) => /^(0|var\(--gap-(tight|snug|base|loose)\)|s\d+)$/.test(v),
  padding: (v) => v.split(/\s+/).every((p) => space.test(p) || /^s\d+$/.test(p)),
  "border-radius": (v) => v.split(/\s+/).every((p) => /^(0|var\(--(radius-(sm|md|lg|pill|sheet)|card-radius|dialog-radius)\))$/.test(p)),
  "font-size": (v) => /^(var\(--text-(xs|sm|md|lg|xl)\)|\d+)$/.test(v) || /^(synopsis-size|card-title|card-subtitle)$/.test(v),
};

const bad = rules.filter((r) => checks[r.prop] && !checks[r.prop](r.value));
if (bad.length) {
  console.error(`${bad.length} values outside the skin scale (use --gap-*, --space-N, --radius-*, --text-*):`);
  for (const r of bad.slice(0, 40)) console.error(`  ${r.head} { ${r.prop}: ${r.value} }`);
  process.exit(1);
}
console.log("skin spacing, radius and text sizes use the shared scale");
