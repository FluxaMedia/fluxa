import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const desktopRoot = process.cwd();
const repoRoot = path.resolve(desktopRoot, '../..');
const sourcePath = path.join(repoRoot, 'shared/contracts/ui-tokens.json');
const source = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));
const themes = source.themes ?? [];
const { themes: _themes, ...uiTokenSource } = source;
const kmpRoot = path.resolve(desktopRoot, '../android');

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
const kotlinThemePath = path.join(
  kmpRoot,
  'shared/src/commonMain/kotlin/com/fluxa/app/ui/catalog/FluxaThemePackDefaults.generated.kt',
);
const swiftThemePath = path.join(kmpRoot, '../apple/tvOS/FluxaThemePackDefaults.generated.swift');

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

const typescript = `export const FLUXA_UI_TOKENS = ${JSON.stringify(uiTokenSource, null, 2)} as const;
`;

const kotlinString = (value) => JSON.stringify(String(value));
const swiftString = (value) => JSON.stringify(String(value));
const kotlinTheme = (theme, propertyName) => {
  const colors = Object.entries(theme.colors)
    .map(([key, value]) => `            ${key} = ${kotlinString(value)},`)
    .join('\n');
  return `    val ${propertyName} = FluxaThemePack(
        schemaVersion = ${theme.schemaVersion},
        id = ${kotlinString(theme.id)},
        nameKey = ${kotlinString(theme.nameKey)},
        colors = FluxaThemeColors(
${colors}
        ),
        typography = FluxaThemeTypography(${kotlinString(theme.typography.displayFont)}, ${kotlinString(theme.typography.bodyFont)}, ${theme.typography.titleWeight}, ${theme.typography.bodyWeight}),
        shape = FluxaThemeShape(${theme.shape.cardRadius}, ${theme.shape.controlRadius}, ${theme.shape.dialogRadius}),
        spacing = FluxaThemeSpacing(${theme.spacing.screenPadding}, ${theme.spacing.sectionGap}, ${theme.spacing.controlGap}),
        motion = FluxaThemeMotion(${theme.motion.enabled}, ${theme.motion.fastMs}, ${theme.motion.normalMs}, ${theme.motion.slowMs}),
        layouts = FluxaThemeLayouts(${kotlinString(theme.layouts.home)}, ${kotlinString(theme.layouts.detail)}, ${kotlinString(theme.layouts.library)}, ${kotlinString(theme.layouts.navigation)}),
    )`;
};

const swiftTheme = (theme, propertyName) => {
  const colors = Object.entries(theme.colors)
    .map(([key, value]) => `            ${key}: ${swiftString(value)},`)
    .join('\n');
  return `    static let ${propertyName} = FluxaThemePack(
        schemaVersion: ${theme.schemaVersion},
        id: ${swiftString(theme.id)},
        nameKey: ${swiftString(theme.nameKey)},
        colors: FluxaThemeColors(
${colors}
        ),
        typography: FluxaThemeTypography(displayFont: ${swiftString(theme.typography.displayFont)}, bodyFont: ${swiftString(theme.typography.bodyFont)}, titleWeight: ${theme.typography.titleWeight}, bodyWeight: ${theme.typography.bodyWeight}),
        shape: FluxaThemeShape(cardRadius: ${theme.shape.cardRadius}, controlRadius: ${theme.shape.controlRadius}, dialogRadius: ${theme.shape.dialogRadius}),
        spacing: FluxaThemeSpacing(screenPadding: ${theme.spacing.screenPadding}, sectionGap: ${theme.spacing.sectionGap}, controlGap: ${theme.spacing.controlGap}),
        motion: FluxaThemeMotion(enabled: ${theme.motion.enabled}, fastMs: ${theme.motion.fastMs}, normalMs: ${theme.motion.normalMs}, slowMs: ${theme.motion.slowMs}),
        layouts: FluxaThemeLayouts(home: ${swiftString(theme.layouts.home)}, detail: ${swiftString(theme.layouts.detail)}, library: ${swiftString(theme.layouts.library)}, navigation: ${swiftString(theme.layouts.navigation)})
    )`;
};

const propertyNames = ['fluxaDark', 'amoled', 'midnightBlue'];
const kotlinThemes = `package com.fluxa.app.ui.catalog

// Generated from shared/contracts/ui-tokens.json. Do not edit.
object FluxaThemePackDefaults {
${themes.map((theme, index) => kotlinTheme(theme, propertyNames[index] ?? theme.id.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()))).join('\n\n')}
}
`;
const swiftThemes = `import Foundation

// Generated from shared/contracts/ui-tokens.json. Do not edit.
enum FluxaThemePackDefaults {
${themes.map((theme, index) => swiftTheme(theme, propertyNames[index] ?? theme.id.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()))).join('\n\n')}
}
`;

const outputs = [
  [kotlinPath, kotlin],
  [kotlinLayoutPath, layoutKotlin],
  [swiftPath, swift],
  [swiftLayoutPath, layoutSwift],
  [tsPath, typescript],
  [cssLayoutPath, layoutCss],
  [kotlinThemePath, kotlinThemes],
  [swiftThemePath, swiftThemes],
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
