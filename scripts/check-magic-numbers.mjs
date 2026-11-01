import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const uiRoot = path.join(root, "native", "fluxa-ui", "src");
const skip = /(^|\/)(tests|scroll_physics|motion|effects|ambient)\.rs$/;
const number = String.raw`-?\d+(?:\.\d+)?(?:_f32)?`;
const patterns = [
  new RegExp(
    String.raw`if !?[\w.()]*(?:compact|tv|desktop|mobile|roomy)[\w.()]*(?: && !?[\w.()]+)? \{ *${number} *\} else \{ *${number} *\}`,
    "g",
  ),
  new RegExp(String.raw`\* 0\.\d+[^;\n]*\)\.clamp\((?!0\.0, 1\.0\))${number}, *${number}\)`, "g"),
];

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (entry.name.endsWith(".rs") && !skip.test(full)) out.push(full);
  }
  return out;
}

const counts = {};
for (const file of walk(uiRoot)) {
  const source = fs.readFileSync(file, "utf8");
  let total = 0;
  for (const pattern of patterns) total += (source.match(pattern) || []).length;
  if (total > 0) counts[path.relative(root, file)] = total;
}

const failures = [];
for (const [file, count] of Object.entries(counts)) {
  failures.push(`${file}: ${count} inline form-factor or ratio sizes; move them into ui-tokens.json`);
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("no new inline form-factor or ratio sizes");
