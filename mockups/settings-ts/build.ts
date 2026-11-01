import { readFileSync, writeFileSync } from "node:fs";

const built = await Bun.build({ entrypoints: ["src/main.ts"], target: "browser", minify: false });
if (!built.success) {
  console.error(built.logs.join("\n"));
  process.exit(1);
}
const js = await built.outputs[0].text();
const css = readFileSync("style.css", "utf8");
const html = readFileSync("index.html", "utf8")
  .replace('<link rel="stylesheet" href="style.css">', () => `<style>\n${css}</style>`)
  .replace('<script src="dist/main.js"></script>', () => `<script>\n${js.replace(/<\/script/g, "<\\/script")}</script>`);
writeFileSync("dist/index.html", html);
