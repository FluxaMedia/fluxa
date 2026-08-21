import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const desktopRoot = process.cwd();
const kmpRoot = process.argv[2] ?? path.resolve(desktopRoot, '../fluxa');
const themePath = path.join(desktopRoot, 'contracts/default-theme.json');
const theme = JSON.parse(fs.readFileSync(themePath, 'utf8'));

const kotlinString = (value) => JSON.stringify(String(value));
const swiftString = (value) => JSON.stringify(String(value));

const kotlinColors = Object.entries(theme.colors)
  .map(([key, value]) => `            ${key} = ${kotlinString(value)},`)
  .join('\n');
const kotlin = `package com.fluxa.app.ui.catalog

object FluxaThemePackDefaults {
    val fluxaDark = FluxaThemePack(
        schemaVersion = ${theme.schemaVersion},
        id = ${kotlinString(theme.id)},
        nameKey = ${kotlinString(theme.nameKey)},
        colors = FluxaThemeColors(
${kotlinColors}
        ),
        typography = FluxaThemeTypography(${kotlinString(theme.typography.displayFont)}, ${kotlinString(theme.typography.bodyFont)}, ${theme.typography.titleWeight}, ${theme.typography.bodyWeight}),
        shape = FluxaThemeShape(${theme.shape.cardRadius}, ${theme.shape.controlRadius}, ${theme.shape.dialogRadius}),
        spacing = FluxaThemeSpacing(${theme.spacing.screenPadding}, ${theme.spacing.sectionGap}, ${theme.spacing.controlGap}),
        motion = FluxaThemeMotion(${theme.motion.enabled}, ${theme.motion.fastMs}, ${theme.motion.normalMs}, ${theme.motion.slowMs}),
        layouts = FluxaThemeLayouts(${kotlinString(theme.layouts.home)}, ${kotlinString(theme.layouts.detail)}, ${kotlinString(theme.layouts.library)}, ${kotlinString(theme.layouts.navigation)}),
    )
}
`;

const swiftColors = Object.entries(theme.colors)
  .map(([key, value]) => `            ${key}: ${swiftString(value)},`)
  .join('\n');
const swift = `import Foundation

enum FluxaThemePackDefaults {
    static let fluxaDark = FluxaThemePack(
        schemaVersion: ${theme.schemaVersion},
        id: ${swiftString(theme.id)},
        nameKey: ${swiftString(theme.nameKey)},
        colors: FluxaThemeColors(
${swiftColors}
        ),
        typography: FluxaThemeTypography(displayFont: ${swiftString(theme.typography.displayFont)}, bodyFont: ${swiftString(theme.typography.bodyFont)}, titleWeight: ${theme.typography.titleWeight}, bodyWeight: ${theme.typography.bodyWeight}),
        shape: FluxaThemeShape(cardRadius: ${theme.shape.cardRadius}, controlRadius: ${theme.shape.controlRadius}, dialogRadius: ${theme.shape.dialogRadius}),
        spacing: FluxaThemeSpacing(screenPadding: ${theme.spacing.screenPadding}, sectionGap: ${theme.spacing.sectionGap}, controlGap: ${theme.spacing.controlGap}),
        motion: FluxaThemeMotion(enabled: ${theme.motion.enabled}, fastMs: ${theme.motion.fastMs}, normalMs: ${theme.motion.normalMs}, slowMs: ${theme.motion.slowMs}),
        layouts: FluxaThemeLayouts(home: ${swiftString(theme.layouts.home)}, detail: ${swiftString(theme.layouts.detail)}, library: ${swiftString(theme.layouts.library)}, navigation: ${swiftString(theme.layouts.navigation)})
    )
}
`;

const kotlinPath = path.join(kmpRoot, 'shared/src/commonMain/kotlin/com/fluxa/app/ui/catalog/FluxaThemePackDefaults.generated.kt');
const swiftPath = path.join(kmpRoot, 'appleApp/tvOS/FluxaThemePackDefaults.generated.swift');
fs.mkdirSync(path.dirname(kotlinPath), { recursive: true });
fs.mkdirSync(path.dirname(swiftPath), { recursive: true });
fs.writeFileSync(kotlinPath, kotlin);
fs.writeFileSync(swiftPath, swift);
