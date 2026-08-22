import { readFileSync } from 'node:fs';
import { globSync } from 'node:fs';

const MIGRATED = ['src/components/detail/**/*.{ts,tsx}'];

const ALLOWED_RADIUS = new Set(['0.25rem', '0.375rem', '0.5rem', '0.75rem', '1rem', '1.25rem', '999px', '50%', '0']);

const ALLOWED_FONT_SIZE = new Set([
  '0.625rem',
  '0.6875rem',
  '0.75rem',
  '0.8125rem',
  '0.875rem',
  '1rem',
  '1.125rem',
  '1.375rem',
  '2rem',
  '3.125rem',
  'inherit',
]);

const isFluid = (value) => value.startsWith('clamp(');

const ALLOWED_Z = new Set(['0', '1', '5', '20', '40', '46', '60', '70', '80', '100']);

const rules = [
  {
    name: 'ham renk',
    re: /#[0-9A-Fa-f]{3,8}\b|\brgba?\([^)]*\)/g,
    hint: 'color.* kullan',
    skip: (m) => m.startsWith('rgba(0,0,0,0)') || m === 'rgb(0,0,0)',
  },
  {
    name: 'ölçek dışı borderRadius',
    re: /borderRadius:\s*'([^']+)'/g,
    hint: 'radius.* kullan',
    value: true,
    allowed: ALLOWED_RADIUS,
  },
  {
    name: 'ölçek dışı fontSize',
    re: /fontSize:\s*'([^']+)'/g,
    hint: 'fontSize.* kullan',
    value: true,
    allowed: ALLOWED_FONT_SIZE,
  },
  {
    name: 'serbest zIndex',
    re: /zIndex:\s*([1-9]\d+)/g,
    hint: 'z.* kullan',
    value: true,
    allowed: ALLOWED_Z,
  },
];

const files = MIGRATED.flatMap((pattern) => globSync(pattern));
let failures = 0;

for (const file of files) {
  const source = readFileSync(file, 'utf8');
  const lines = source.split('\n');

  lines.forEach((line, index) => {
    for (const rule of rules) {
      rule.re.lastIndex = 0;
      let match;
      while ((match = rule.re.exec(line)) !== null) {
        const captured = rule.value ? match[1] : match[0];
        if (rule.value && rule.allowed.has(captured)) continue;
        if (rule.value && isFluid(captured)) continue;
        if (!rule.value && rule.skip?.(captured)) continue;
        failures += 1;
        console.log(`${file}:${index + 1}  ${rule.name}: ${captured}  →  ${rule.hint}`);
      }
    }
  });
}

if (failures > 0) {
  console.log(`\n${failures} ihlal. Kurallar: DESIGN.md`);
  process.exit(1);
}

console.log(`${files.length} dosya temiz.`);
