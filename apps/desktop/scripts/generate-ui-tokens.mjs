import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const desktopRoot = process.cwd();
const repoRoot = path.resolve(desktopRoot, '../..');
const sourcePath = path.join(repoRoot, 'shared/contracts/ui-tokens.json');
const source = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));

const kotlinPath = path.join(
  repoRoot,
  'apps/android/shared/src/commonMain/kotlin/com/fluxa/app/ui/catalog/FluxaUiTokens.generated.kt',
);
const kotlinLayoutPath = path.join(
  repoRoot,
  'apps/android/shared/src/commonMain/kotlin/com/fluxa/app/ui/catalog/FluxaUiLayoutTokens.generated.kt',
);
const swiftPath = path.join(repoRoot, 'apps/apple/tvOS/FluxaUiTokens.generated.swift');
const swiftLayoutPath = path.join(repoRoot, 'apps/apple/tvOS/FluxaUiLayoutTokens.generated.swift');
const tsPath = path.join(desktopRoot, 'src/theme/uiTokens.generated.ts');
const cssLayoutPath = path.join(desktopRoot, 'src/theme/uiLayoutTokens.generated.css');

const json = (value) => JSON.stringify(value);
const colorToKotlin = (value) => {
  const hex = value.slice(1);
  const argb = hex.length === 6 ? `FF${hex}` : `${hex.slice(6)}${hex.slice(0, 6)}`;
  return `Color(0x${argb.toUpperCase()})`;
};
const swiftColor = (value) => json(value);
const upperFirst = (value) => value[0].toUpperCase() + value.slice(1);
const cssName = (value) => value.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);

const emitKotlinEntries = (entries, unit) =>
  Object.entries(entries ?? {})
    .map(([key, value]) => {
      if (unit === 'dp') return `        val ${key} = ${value}.dp`;
      if (unit === 'sp') return `        val ${key} = ${value}.sp`;
      if (unit === 'durationMs') return `        const val ${key} = ${value}`;
      if (unit === 'int') return `        const val ${key} = ${value}`;
      return `        const val ${key} = ${value}f`;
    })
    .join('\n');

const emitKotlinSection = (name, section) => {
  const units = [
    ['Dp', 'dp'],
    ['Sp', 'sp'],
    ['DurationMs', 'durationMs'],
    ['Number', 'number'],
    ['Alpha', 'alpha'],
    ['Int', 'int'],
  ];
  const body = units
    .filter(([, key]) => section?.[key] && Object.keys(section[key]).length > 0)
    .map(([typeName, key]) => `    object ${typeName} {\n${emitKotlinEntries(section[key], key)}\n    }`)
    .join('\n');
  return `  object ${name} {\n${body}\n  }`;
};

const emitSwiftEntries = (entries, unit) =>
  Object.entries(entries ?? {})
    .map(([key, value]) => {
      if (unit === 'durationMs' || unit === 'int') return `        static let ${key} = ${value}`;
      return `        static let ${key} = CGFloat(${value})`;
    })
    .join('\n');

const emitSwiftSection = (name, section) => {
  const units = [
    ['Dp', 'dp'],
    ['Sp', 'sp'],
    ['DurationMs', 'durationMs'],
    ['Number', 'number'],
    ['Alpha', 'alpha'],
    ['Int', 'int'],
  ];
  const body = units
    .filter(([, key]) => section?.[key] && Object.keys(section[key]).length > 0)
    .map(([typeName, key]) => `    enum ${typeName} {\n${emitSwiftEntries(section[key], key)}\n    }`)
    .join('\n');
  return `  enum ${name} {\n${body}\n  }`;
};

const emitActiveCssEntries = (entries, unit) =>
  Object.entries(entries ?? {})
    .map(([key, value]) => {
      const suffix = unit === 'dp' || unit === 'sp' ? 'px' : unit === 'durationMs' ? 'ms' : '';
      return `  --fluxa-layout-${cssName(key)}: ${value}${suffix};`;
    })
    .join('\n');

const layout = source.layout ?? {};
const layoutKotlin = `package com.fluxa.app.ui.catalog

import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// Generated from shared/contracts/ui-tokens.json. Do not edit.
object FluxaUiLayoutTokens {
${emitKotlinSection('Common', layout.common)}
${Object.entries(layout.platforms ?? {})
  .map(([name, section]) => emitKotlinSection(upperFirst(name), section))
  .join('\n')}
${Object.entries(layout.windowClasses ?? {})
  .map(([name, section]) => emitKotlinSection(upperFirst(name), section))
  .join('\n')}
${emitKotlinSection('MaterialTheme', layout.materialTheme)}
}
`;

const layoutSwift = `import Foundation

// Generated from shared/contracts/ui-tokens.json. Do not edit.
enum FluxaUiLayoutTokens {
${emitSwiftSection('Common', layout.common)}
${Object.entries(layout.platforms ?? {})
  .map(([name, section]) => emitSwiftSection(upperFirst(name), section))
  .join('\n')}
${Object.entries(layout.windowClasses ?? {})
  .map(([name, section]) => emitSwiftSection(upperFirst(name), section))
  .join('\n')}
${emitSwiftSection('MaterialTheme', layout.materialTheme)}
}
`;

const cssSections = [];
if (layout.common) {
  for (const [unit, values] of Object.entries(layout.common)) {
    cssSections.push(emitActiveCssEntries(values, unit));
  }
}
const platformCss = Object.entries(layout.platforms ?? {})
  .map(([name, section]) => {
    const declarations = Object.entries(section)
      .map(([unit, values]) => emitActiveCssEntries(values, unit))
      .join('\n');
    return `[data-fluxa-platform="${name}"] {\n${declarations}\n}`;
  })
  .join('\n');
const defaultDesktop = Object.entries(layout.platforms?.desktop ?? {})
  .map(([unit, values]) => emitActiveCssEntries(values, unit))
  .join('\n');
const windowCss = Object.entries(layout.windowClasses ?? {})
  .map(([name, section]) => {
    const declarations = Object.entries(section)
      .map(([unit, values]) => emitActiveCssEntries(values, unit))
      .join('\n');
    return `[data-fluxa-window-class="${name}"] {\n${declarations}\n}`;
  })
  .join('\n');
const layoutCss = `/* Generated from shared/contracts/ui-tokens.json. Do not edit. */
:root {
${defaultDesktop}
${cssSections.join('\n')}
}
${platformCss}
${windowCss}
`;

const kotlin = `package com.fluxa.app.ui.catalog

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

object FluxaUiTokens {
    const val schemaVersion = ${source.schemaVersion}
${Object.entries(source.colors).map(([key, value]) => `    val color${key[0].toUpperCase()}${key.slice(1)} = ${colorToKotlin(value)}`).join('\n')}
${Object.entries(source.shape).map(([key, value]) => `    val shape${key[0].toUpperCase()}${key.slice(1)} = ${value}.dp`).join('\n')}
${Object.entries(source.spacing).map(([key, value]) => `    val spacing${key[0].toUpperCase()}${key.slice(1)} = ${value}.dp`).join('\n')}
${Object.entries(source.auth).map(([key, value]) => `    val auth${key[0].toUpperCase()}${key.slice(1)} = ${value}.dp`).join('\n')}
${Object.entries(source.typography).map(([key, value]) => `    val typography${key[0].toUpperCase()}${key.slice(1)} = ${value}.sp`).join('\n')}
}
`;

const swift = `import Foundation

enum FluxaUiTokens {
    static let schemaVersion = ${source.schemaVersion}
${Object.entries(source.colors).map(([key, value]) => `    static let color${key[0].toUpperCase()}${key.slice(1)} = ${swiftColor(value)}`).join('\n')}
${[...Object.entries(source.shape), ...Object.entries(source.spacing), ...Object.entries(source.auth)].map(([key, value]) => `    static let ${key} = CGFloat(${value})`).join('\n')}
${Object.entries(source.typography).map(([key, value]) => `    static let typography${key[0].toUpperCase()}${key.slice(1)} = CGFloat(${value})`).join('\n')}
}
`;

const typescript = `export const FLUXA_UI_TOKENS = ${JSON.stringify(source, null, 2)} as const;
`;

const outputs = [
  [kotlinPath, kotlin],
  [kotlinLayoutPath, layoutKotlin],
  [swiftPath, swift],
  [swiftLayoutPath, layoutSwift],
  [tsPath, typescript],
  [cssLayoutPath, layoutCss],
];

if (process.argv.includes('--check')) {
  const stale = outputs.filter(([filePath, content]) => !fs.existsSync(filePath) || fs.readFileSync(filePath, 'utf8') !== content);
  if (stale.length > 0) throw new Error(`stale generated UI token consumers: ${stale.map(([filePath]) => filePath).join(', ')}`);
  process.stdout.write(`verified ${outputs.length} generated UI token consumers\n`);
} else {
  for (const [filePath, content] of outputs) {
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, content);
  }
  process.stdout.write(`generated ${outputs.length} UI token consumers\n`);
}
