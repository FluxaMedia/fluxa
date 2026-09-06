import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

const root = process.cwd();
const manifestPath = path.join(root, 'core', 'fluxa-core', 'settings', 'settings-manifest.json');
const desktopSettingsPath = path.join(root, 'apps', 'desktop', 'src', 'components', 'settings', 'settingsTypes.ts');
const desktopGeneratedSettingsPath = path.join(root, 'apps', 'desktop', 'src', 'core', 'generated', 'settingsContract.ts');
const androidSettingsPath = path.join(
  root,
  'apps',
  'android',
  'shared',
  'src',
  'commonMain',
  'kotlin',
  'com',
  'fluxa',
  'app',
  'shared',
  'feature',
  'settings',
  'PersistentSettingsDataSource.kt',
);

const desktopSource = fs.readFileSync(desktopSettingsPath, 'utf8');
const androidSource = fs.readFileSync(androidSettingsPath, 'utf8');

function parseDesktopDefaults() {
  const legacyMatch = desktopSource.match(/export const DEFAULT_PREFS: Prefs = (\{[\s\S]*?\n\});/);
  if (legacyMatch) return vm.runInNewContext(`(${legacyMatch[1]})`);
  if (!fs.existsSync(desktopGeneratedSettingsPath)) throw new Error('Desktop generated settings defaults not found');
  const generatedSource = fs.readFileSync(desktopGeneratedSettingsPath, 'utf8');
  const generatedMatch = generatedSource.match(/export const SETTINGS_DEFAULTS[^=]*= (\{[\s\S]*?\n\});/);
  if (!generatedMatch) throw new Error('Desktop generated settings defaults object not found');
  return vm.runInNewContext(`(${generatedMatch[1]})`);
}

function parseDesktopTypes() {
  const match = desktopSource.match(/export interface Prefs \{([\s\S]*?)\n\}/);
  if (!match) throw new Error('Desktop Prefs interface not found');
  return new Map(
    [...match[1].matchAll(/^\s+([A-Za-z0-9_]+):\s+([^;]+);/gm)].map(([, key, type]) => [key, type.trim()]),
  );
}

function parseAndroidKeys() {
  return [...androidSource.matchAll(/const val\s+([A-Z0-9_]+)\s*=\s*"([^"]+)"/g)].map(([, name, key]) => ({
    name,
    key,
    type: inferAndroidType(name),
  }));
}

function inferAndroidType(name) {
  const key = `SettingsPreferenceKeys.${name}`;
  if (new RegExp(`getBoolean\\(${key}\\b`).test(androidSource) || new RegExp(`putBoolean\\(${key}\\b`).test(androidSource)) return 'boolean';
  if (new RegExp(`getInt\\(${key}\\b`).test(androidSource) || new RegExp(`putInt\\(${key}\\b`).test(androidSource)) return 'number';
  if (new RegExp(`getLong\\(${key}\\b`).test(androidSource) || new RegExp(`putLong\\(${key}\\b`).test(androidSource)) return 'number';
  if (new RegExp(`getFloat\\(${key}\\b`).test(androidSource) || new RegExp(`putFloat\\(${key}\\b`).test(androidSource)) return 'number';
  return 'string';
}

function desktopType(type) {
  if (type === 'boolean') return 'boolean';
  if (type === 'string[]') return 'string[]';
  if (type === 'number') return 'number';
  return 'string';
}

function categoryFor(key) {
  if (/^(tmdb|rpdb|omdb|mdblist|fanart)/.test(key)) return 'account';
  if (/^(subtitle|preferredAudio|secondaryAudio|autoEnableSubtitles)/.test(key)) return 'subtitles';
  if (/^(download)/.test(key)) return 'downloads';
  if (/^(theme|accent|skin|customThemes|uiScale|animations|reducedEffects|gif|nav|interface|poster|card|catalog|continueWatching|trailerOn|blurUnwatched|spoiler|showHero|homeHero|detailHero|heroFeed|homeFeed|topTenFeed|detailSeason|episodeCards)/.test(key)) return 'appearance';
  if (/^(preferredPlayer|externalPlayer|mpv|anime|frameInterpolation|stream|autoRetry|autoPlay|tryBinge|nextEpisode|watched|movieRecommendation|seriesRecommendation|playback|seek|hold|audioProcessing|stableVolume|player|autoSkip|useSkip|introDb|theIntroDb|animeSkip|p2p|renderBackend|hdr|playerEngine)/.test(key)) return 'playback';
  if (/^(continueWatching|syncCw|similarTitles|integrationLibrary|watchProgress|traktComments)/.test(key)) return 'library';
  if (/^(notification|alert|automaticUpdates|timezone|pictureInPicture|discord|diagnostic|backgroundPlayback|startPage|language)/.test(key)) return 'system';
  return 'general';
}

function bootstrapManifest() {
  const defaults = parseDesktopDefaults();
  const types = parseDesktopTypes();
  const androidKeys = parseAndroidKeys();
  const entries = new Map();

  for (const [key, value] of Object.entries(defaults)) {
    entries.set(key, {
      key,
      type: desktopType(types.get(key) ?? typeof value),
      default: value,
      scope: 'profile',
      category: categoryFor(key),
    });
  }

  for (const { key, type } of androidKeys) {
    const current = entries.get(key);
    if (current) continue;
    entries.set(key, {
      key,
      type,
      default: null,
      scope: 'profile',
      category: categoryFor(key),
    });
  }

  return {
    version: 1,
    settings: [...entries.values()].sort((a, b) => a.key.localeCompare(b.key)),
  };
}

function upperSnake(key) {
  return key.replace(/([a-z0-9])([A-Z])/g, '$1_$2').replace(/[^A-Za-z0-9]+/g, '_').toUpperCase();
}

function camelIdentifier(key) {
  return key.replace(/[^A-Za-z0-9]+(.)/g, (_, character) => character.toUpperCase());
}

function tsType(entry) {
  if (entry.default === null) return 'unknown';
  if (entry.type === 'boolean') return 'boolean';
  if (entry.type === 'number') return 'number';
  if (entry.type === 'string[]') return 'string[]';
  return 'string';
}

function kotlinConstant(entry) {
  return `    const val ${upperSnake(entry.key)} = "${entry.key}"`;
}

function swiftIdentifier(key) {
  return key.replace(/[^A-Za-z0-9]+(.)/g, (_, character) => character.toUpperCase());
}

function renderOutputs(manifest) {
  const entries = manifest.settings;
  const settingsKeys = entries.map((entry) => `  ${camelIdentifier(entry.key)}: '${entry.key}',`).join('\n');
  const defaults = entries
    .filter((entry) => entry.default !== null)
    .map((entry) => `  ${camelIdentifier(entry.key)}: ${JSON.stringify(entry.default)},`)
    .join('\n');
  const generatedTypes = entries
    .map((entry) => `  ${camelIdentifier(entry.key)}: ${tsType(entry)};`)
    .join('\n');
  const kotlinKeys = entries.map(kotlinConstant).join('\n');
  const swiftKeys = entries.map((entry) => `    static let ${swiftIdentifier(entry.key)} = "${entry.key}"`).join('\n');

  return new Map([
    [
      path.join(root, 'apps', 'desktop', 'src', 'core', 'generated', 'settingsContract.ts'),
      `// Generated by scripts/generate-settings-contract.mjs — do not edit.\nexport const SETTINGS_KEYS = {\n${settingsKeys}\n} as const;\n\nexport type SettingKey = (typeof SETTINGS_KEYS)[keyof typeof SETTINGS_KEYS];\n\nexport interface GeneratedSettingsValues {\n${generatedTypes}\n}\n\nexport const SETTINGS_DEFAULTS: Partial<GeneratedSettingsValues> = {\n${defaults}\n};\n`,
    ],
    [
      path.join(
        root,
        'apps',
        'android',
        'shared',
        'src',
        'commonMain',
        'kotlin',
        'com',
        'fluxa',
        'app',
        'shared',
        'feature',
        'settings',
        'GeneratedSettingsPreferenceKeys.kt',
      ),
      `// Generated by scripts/generate-settings-contract.mjs — do not edit.\npackage com.fluxa.app.shared.feature.settings\n\nobject GeneratedSettingsPreferenceKeys {\n${kotlinKeys}\n}\n`,
    ],
    [
      path.join(root, 'apps', 'apple', 'AppleCore', 'Generated', 'FluxaSettingsKeys.swift'),
      `// Generated by scripts/generate-settings-contract.mjs — do not edit.\nenum FluxaSettingsKey {\n${swiftKeys}\n}\n`,
    ],
  ]);
}

function writeOrCheck(outputs, checkOnly) {
  for (const [outputPath, content] of outputs) {
    const current = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : '';
    if (current === content) continue;
    if (checkOnly) throw new Error(`generated settings contract is out of date: ${path.relative(root, outputPath)}`);
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    fs.writeFileSync(outputPath, content);
  }
}

const checkOnly = process.argv.includes('--check');
const bootstrap = process.argv.includes('--bootstrap');

if (bootstrap && fs.existsSync(manifestPath)) {
  throw new Error(`${path.relative(root, manifestPath)} already exists; remove --bootstrap to regenerate outputs`);
}

const manifest = fs.existsSync(manifestPath) ? JSON.parse(fs.readFileSync(manifestPath, 'utf8')) : bootstrapManifest();
if (!Array.isArray(manifest.settings) || manifest.settings.length === 0) throw new Error('settings manifest is empty');

const keys = manifest.settings.map((entry) => entry.key);
if (new Set(keys).size !== keys.length) throw new Error('duplicate setting key in manifest');
if (!keys.every((key) => /^[A-Za-z][A-Za-z0-9]*$/.test(key))) throw new Error('setting keys must be camelCase identifiers');

if (checkOnly) {
  const manifestByKey = new Map(manifest.settings.map((entry) => [entry.key, entry]));
  const desktopDefaults = parseDesktopDefaults();
  const desktopTypes = parseDesktopTypes();
  for (const [key, value] of Object.entries(desktopDefaults)) {
    const entry = manifestByKey.get(key);
    if (!entry) throw new Error(`desktop setting is missing from manifest: ${key}`);
    if (JSON.stringify(entry.default) !== JSON.stringify(value)) throw new Error(`desktop default drift: ${key}`);
    if (entry.type !== desktopType(desktopTypes.get(key) ?? typeof value)) throw new Error(`desktop type drift: ${key}`);
  }
  for (const { key } of parseAndroidKeys()) {
    if (!manifestByKey.has(key)) throw new Error(`Android setting is missing from manifest: ${key}`);
  }
}

if (bootstrap) {
  fs.mkdirSync(path.dirname(manifestPath), { recursive: true });
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}

writeOrCheck(renderOutputs(manifest), checkOnly);

if (!checkOnly) process.stdout.write(`generated ${manifest.settings.length} settings across desktop, Android, and Apple contracts\n`);
