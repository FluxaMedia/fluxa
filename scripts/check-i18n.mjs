import fs from "node:fs";
import path from "node:path";

const directory = path.join(process.cwd(), "shared", "i18n");
const languages = ["english_us", "tr_tr", "ar", "he"];
const files = Object.fromEntries(
  languages.map((language) => [
    language,
    JSON.parse(fs.readFileSync(path.join(directory, `${language}.json`), "utf8")),
  ]),
);
const problems = [];
for (const from of languages) {
  for (const to of languages) {
    if (from === to) continue;
    for (const key of Object.keys(files[from])) {
      if (!(key in files[to])) problems.push(`${to} is missing ${key}`);
    }
  }
}
if (problems.length > 0) {
  throw new Error(`shared i18n keys differ:\n${[...new Set(problems)].join("\n")}`);
}
