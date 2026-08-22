import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const desktopRoot = process.cwd();
const kmpRoot = process.argv[2] ?? path.resolve(desktopRoot, '../android');
const methodsPath = path.join(desktopRoot, 'src/core/coreMethods.ts');
const androidBridgePath = path.join(
  kmpRoot,
  'data/src/jvmCommonMain/kotlin/com/fluxa/app/core/rust/FluxaCoreNative.kt',
);

const methodsSource = fs.readFileSync(methodsPath, 'utf8');
const androidSource = fs.readFileSync(androidBridgePath, 'utf8');
const knownMethods = new Set(
  [...methodsSource.matchAll(/^  '([^']+)',$/gm)].map((match) => match[1]),
);
const androidMethods = [
  ...androidSource.matchAll(/coreInvokeValue\("([A-Za-z][A-Za-z0-9.]*)"/g),
].map((match) => match[1]);
const missing = [...new Set(androidMethods)].filter((method) => !knownMethods.has(method));

if (missing.length > 0) {
  throw new Error(`Android core consumer references unknown methods: ${missing.join(', ')}`);
}

process.stdout.write(
  `verified ${new Set(androidMethods).size} Android core methods against ${knownMethods.size} generated methods\n`,
);
