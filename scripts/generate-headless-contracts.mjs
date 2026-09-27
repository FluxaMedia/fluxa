import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const rustPath = path.join(root, 'core', 'fluxa-core', 'src', 'runtime', 'effects.rs');
const source = fs.readFileSync(rustPath, 'utf8');
const effects = [...source.matchAll(/EffectKind::([A-Za-z0-9_]+) => "([A-Za-z0-9.]+)"/g)].map((match) => ({
  variant: match[1],
  value: match[2],
}));

if (effects.length === 0) throw new Error('No EffectKind mappings found');
if (new Set(effects.map(({ value }) => value)).size !== effects.length) {
  throw new Error('Duplicate headless effect value found');
}

const actionRustPath = path.join(root, 'core', 'fluxa-core', 'src', 'headless_engine', 'contracts.rs');
const actionSource = fs.readFileSync(actionRustPath, 'utf8');
const actionEnum = actionSource.match(/pub\(super\) enum AppAction \{([\s\S]*?)\n\}\n\n#\[derive/);
if (!actionEnum) throw new Error('AppAction enum not found');
const actions = [...actionEnum[1].matchAll(/#\[serde\(rename = "([^"]+)"\)\]\s*\n\s*[A-Z][A-Za-z0-9_]*/g)].map((match) => match[1]);
if (actions.length === 0) throw new Error('No AppAction mappings found');
if (new Set(actions).size !== actions.length) throw new Error('Duplicate headless action value found');

const checkOnly = process.argv.includes('--check');
const outputs = new Map([
  [
    path.join(root, 'shared', 'contracts', 'headless-effects.json'),
    `${JSON.stringify({ effects }, null, 2)}\n`,
  ],
  [
    path.join(root, 'shared', 'contracts', 'headless-actions.json'),
    `${JSON.stringify({ actions }, null, 2)}\n`,
  ],
]);

for (const [outputPath, content] of outputs) {
  const current = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : '';
  if (current === content) continue;
  if (checkOnly) throw new Error(`generated headless contract is out of date: ${path.relative(root, outputPath)}`);
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, content);
}

if (!checkOnly) process.stdout.write(`generated ${effects.length} headless effect types and ${actions.length} headless action types\n`);
