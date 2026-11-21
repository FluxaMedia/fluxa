import fs from "node:fs";
import { collectClasses, contractPath, isLayoutHelper } from "./skin-classes.mjs";

const contract = JSON.parse(fs.readFileSync(contractPath, "utf8"));
const used = new Set(collectClasses().keys());
const failures = [];
const removed = new Map(contract.removed.map((entry) => [entry.name, entry]));

if (!Number.isInteger(contract.version) || contract.version < 1)
  failures.push("skin-classes.json: version must be a positive integer");

for (const entry of contract.classes) {
  if (entry.stability !== "stable" && entry.stability !== "internal")
    failures.push(`${entry.name}: stability must be stable or internal`);
  if (entry.stability === "stable") {
    if (isLayoutHelper(entry.name))
      failures.push(`${entry.name}: layout helpers cannot be stable`);
    if (!entry.meaning) failures.push(`${entry.name}: stable class needs a meaning`);
    if (!used.has(entry.name))
      failures.push(
        `${entry.name}: stable class is no longer used; move it to "removed" with removedIn above its since (${entry.since}) and bump version`,
      );
  }
}

for (const entry of contract.removed) {
  const entrySince = entry.since ?? 1;
  if (!(entry.removedIn > entrySince))
    failures.push(`${entry.name}: removedIn must be greater than since (${entrySince})`);
  if (entry.removedIn > contract.version)
    failures.push(`${entry.name}: removedIn ${entry.removedIn} is above contract version ${contract.version}`);
  if (contract.classes.some((c) => c.name === entry.name))
    failures.push(`${entry.name}: listed both as class and as removed`);
}

const known = new Set(contract.classes.map((c) => c.name));
const undeclared = [...used].filter((name) => !known.has(name) && !removed.has(name));

if (failures.length > 0)
  throw new Error(`skin class contract check failed:\n${failures.join("\n")}`);

process.stdout.write(
  `skin class contract verified (v${contract.version}, ${undeclared.length} undeclared; run node scripts/skin-classes.mjs)\n`,
);
