import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const desktopRoot = process.cwd();
const kmpRoot = process.argv[2] ?? path.resolve(desktopRoot, '../android');
const theme = JSON.parse(fs.readFileSync(path.join(desktopRoot, 'contracts/built-in-themes.json'), 'utf8'));
const consumers = [
  path.join(kmpRoot, 'shared/src/commonMain/kotlin/com/fluxa/app/ui/catalog/FluxaThemePackDefaults.generated.kt'),
  path.join(kmpRoot, '../apple/tvOS/FluxaThemePackDefaults.generated.swift'),
];

const values = [];
const collect = (value) => {
  if (value && typeof value === 'object') {
    Object.values(value).forEach(collect);
  } else if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    values.push(String(value));
  }
};
collect(theme);

for (const consumerPath of consumers) {
  const source = fs.readFileSync(consumerPath, 'utf8');
  const missing = values.filter((value) => !source.includes(value));
  if (missing.length > 0) throw new Error(`${consumerPath} is missing generated theme values: ${missing.join(', ')}`);
}

process.stdout.write(`verified ${consumers.length} generated theme consumers\n`);
