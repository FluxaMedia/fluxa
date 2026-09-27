import fs from "node:fs";
import path from "node:path";

const directory = path.join(process.cwd(), "shared", "i18n");
const [english, turkish] = ["english_us", "tr_tr"].map((language) =>
  JSON.parse(fs.readFileSync(path.join(directory, `${language}.json`), "utf8")),
);
const missing = (from, to) => Object.keys(from).filter((key) => !(key in to));
const problems = [
  ...missing(english, turkish).map((key) => `tr_tr is missing ${key}`),
  ...missing(turkish, english).map((key) => `english_us is missing ${key}`),
];
if (problems.length > 0) {
  throw new Error(`shared i18n keys differ:\n${problems.join("\n")}`);
}
