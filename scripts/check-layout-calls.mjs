import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const uiRoot = path.join(root, "native", "fluxa-ui", "src");
const baselinePath = path.join(root, "scripts", "layout-calls-baseline.json");
const skip = /(^|\/)(flex|gfx|tests)\.rs$|^gfx\/|(^|\/)kit\/(items|solve)\.rs$/;
const patterns = [
  /\.horizontal(?:_wrapped|_centered)?\(/g,
  /\.vertical(?:_centered|_centered_justified)?\(/g,
  /\.with_layout\(/g,
  /\.columns\(/g,
  /\.add_space\(/g,
  /\ballocate_ui(?:_with_layout|_at_rect)?\(/g,
  /\.cursor\(\)/g,
  /\.next_widget_position\(\)/g,
  /\bvgap\(/g,
  /\bhgap\(/g,
  /\bui\.gap\(/g,
  /\bkit::space\(/g,
];

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (entry.name.endsWith(".rs") && !skip.test(path.relative(uiRoot, full))) out.push(full);
  }
  return out;
}

const rectCall = /(?:Rect::from_(?:min_size|center_size)|\brect_at|\brect_around|\bsquare_around)\(/g;
const relative =
  /\.(?:min|max)\b|\.(?:center|left_top|right_top|left_bottom|right_bottom|left_center|right_center|center_top|center_bottom|left|right|top|bottom|width|height|size|available_rect_before_wrap)\(\)/;

function relativeRects(source) {
  let count = 0;
  for (const match of source.matchAll(rectCall)) {
    let depth = 1;
    let index = match.index + match[0].length;
    const start = index;
    while (index < source.length && depth > 0) {
      const char = source[index];
      if (char === "(") depth += 1;
      else if (char === ")") depth -= 1;
      index += 1;
    }
    if (relative.test(source.slice(start, index - 1))) count += 1;
  }
  return count;
}

const counts = {};
for (const file of walk(uiRoot)) {
  const source = fs.readFileSync(file, "utf8");
  let total = 0;
  for (const pattern of patterns) total += (source.match(pattern) || []).length;
  total += relativeRects(source);
  if (total > 0) counts[path.relative(uiRoot, file)] = total;
}

if (process.argv.includes("--counts")) {
  const filter = process.argv.slice(2).filter((arg) => !arg.startsWith("--"));
  for (const [file, count] of Object.entries(counts).sort(([a], [b]) => a.localeCompare(b)))
    if (filter.length === 0 || filter.some((prefix) => file.startsWith(prefix))) console.log(`${count}\t${file}`);
  process.exit(0);
}

if (process.argv.includes("--update")) {
  const sorted = Object.fromEntries(Object.entries(counts).sort(([a], [b]) => a.localeCompare(b)));
  fs.writeFileSync(baselinePath, `${JSON.stringify(sorted, null, 2)}\n`);
  console.log(`wrote ${Object.keys(sorted).length} files, total ${Object.values(sorted).reduce((a, b) => a + b, 0)}`);
  process.exit(0);
}

const baseline = fs.existsSync(baselinePath) ? JSON.parse(fs.readFileSync(baselinePath, "utf8")) : {};
const failures = [];
for (const [file, count] of Object.entries(counts)) {
  const allowed = baseline[file] ?? 0;
  if (count > allowed)
    failures.push(`native/fluxa-ui/src/${file}: ${count} structural layout calls, baseline ${allowed}; place children with flex (kit::lay) instead`);
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
const total = Object.values(counts).reduce((a, b) => a + b, 0);
console.log(`structural layout calls: ${total}`);
