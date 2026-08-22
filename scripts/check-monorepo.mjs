import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const requiredPaths = [
  "apps/android/settings.gradle.kts",
  "apps/android/gradlew",
  "apps/apple/project.yml",
  "apps/desktop/package.json",
  "apps/desktop/src-tauri/Cargo.toml",
  "core/fluxa-core/Cargo.toml",
  "shared/i18n/english_us.json",
  "shared/i18n/tr_tr.json",
  ".github/workflows",
  ".github/PULL_REQUEST_TEMPLATE.md",
  ".github/ISSUE_TEMPLATE/bug_report.yml",
  ".github/ISSUE_TEMPLATE/feature_request.yml",
  ".github/labels.yml",
  "CONTRIBUTING.md",
  "README.md",
];

const failures = [];
for (const relativePath of requiredPaths) {
  if (!fs.existsSync(path.join(root, relativePath)))
    failures.push(`missing ${relativePath}`);
}

const workflowRoot = path.join(root, ".github", "workflows");
if (fs.existsSync(workflowRoot)) {
  for (const name of fs.readdirSync(workflowRoot)) {
    if (!name.endsWith(".yml") && !name.endsWith(".yaml")) continue;
    const filePath = path.join(workflowRoot, name);
    const source = fs.readFileSync(filePath, "utf8");
    const forbidden = [
      "repository: FluxaMedia/fluxa-core",
      "path: fluxa-core",
      "working-directory: fluxa-core",
      "workspaces: src-tauri",
    ];
    for (const token of forbidden) {
      if (source.includes(token))
        failures.push(
          `${path.relative(root, filePath)} contains stale ${token}`,
        );
    }
  }
}

for (const nested of [
  "apps/android/.github/workflows",
  "apps/desktop/.github/workflows",
  "core/fluxa-core/.github/workflows",
]) {
  if (fs.existsSync(path.join(root, nested)))
    failures.push(`workflow must live at root: ${nested}`);
}

for (const nested of [
  "apps/android/CONTRIBUTING.md",
  "apps/apple/CONTRIBUTING.md",
  "apps/desktop/CONTRIBUTING.md",
  "core/fluxa-core/CONTRIBUTING.md",
]) {
  if (fs.existsSync(path.join(root, nested)))
    failures.push(`contribution rules must live at root: ${nested}`);
}

for (const legacy of [
  "apps/android/core/src/commonMain/resources/i18n",
  "apps/desktop/src/i18n/english_us.json",
  "apps/desktop/src/i18n/tr_tr.json",
]) {
  if (fs.existsSync(path.join(root, legacy)))
    failures.push(`i18n must use shared/i18n: ${legacy}`);
}

if (failures.length > 0) {
  throw new Error(`monorepo structure check failed:\n${failures.join("\n")}`);
}

process.stdout.write("monorepo structure verified\n");
