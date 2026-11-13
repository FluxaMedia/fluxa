import fs from "node:fs";
import path from "node:path";

const hostRoot = path.join(process.cwd(), "native", "fluxa-host", "src");

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else if (entry.name.endsWith(".rs")) out.push(full);
  }
  return out;
}

const failures = [];
for (const file of walk(hostRoot)) {
  fs.readFileSync(file, "utf8").split("\n").forEach((line, index) => {
    if (/std::time::\{[^}]*\bInstant\b|std::time::Instant/.test(line))
      failures.push(`${path.relative(process.cwd(), file)}:${index + 1}: use web_time::Instant so the web build compiles`);
  });
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("fluxa-host uses web_time::Instant");
