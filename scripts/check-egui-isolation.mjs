import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const uiRoot = path.join(root, "native", "fluxa-ui", "src");

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (entry.name.endsWith(".rs")) out.push(full);
  }
  return out;
}

const failures = [];
for (const file of walk(uiRoot)) {
  const rel = path.relative(uiRoot, file);
  if (rel === "gfx.rs" || rel.startsWith(`gfx${path.sep}`)) continue;
  fs.readFileSync(file, "utf8").split("\n").forEach((line, index) => {
    if (/\begui\b/.test(line))
      failures.push(`native/fluxa-ui/src/${rel}:${index + 1}: egui is only allowed inside gfx; use crate::gfx`);
  });
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("egui is confined to fluxa-ui gfx");
