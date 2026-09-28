import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const brand = join(root, "shared/brand");
const { default: defaultId, icons } = JSON.parse(readFileSync(join(brand, "app-icons.json"), "utf8"));
const template = readFileSync(join(brand, "fluxa-mark.svg"), "utf8").trim();
const android = join(root, "apps/android/app/src");
const xcassets = join(root, "apps/apple/Assets.xcassets");

const markSvg = (icon) =>
  template.replace('stop-color="#FF7A2F"', `stop-color="${icon.from}"`).replace('stop-color="#FF2E63"', `stop-color="${icon.to}"`);

const tileSvg = (icon, size, background, scale = 0.7) => {
  const inner = markSvg(icon).replace(/^<svg[^>]*>/, "").replace(/<\/svg>$/, "");
  const s = (size * scale) / 256;
  const offset = (size - size * scale) / 2;
  const fill = background ? `<rect width="${size}" height="${size}" fill="${background}"/>` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}">${fill}<g transform="translate(${offset} ${offset}) scale(${s})">${inner}</g></svg>`;
};

const write = (path, text) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text);
};

const png = (svg, path, { gray = false } = {}) => {
  mkdirSync(dirname(path), { recursive: true });
  const raw = execFileSync("rsvg-convert", ["-f", "png"], { input: svg });
  const args = gray ? ["png:-", "-colorspace", "Gray", path] : ["png:-", path];
  execFileSync("magick", args, { input: raw });
};

const argb = (hex) => `#FF${hex.slice(1).toUpperCase()}`;

const vector = (icon) => `<vector xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:aapt="http://schemas.android.com/aapt"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="256"
    android:viewportHeight="256">
    <group android:translateX="-6">
        <clip-path android:pathData="M86,46 C70,37 52,48 52,66 V190 C52,208 70,219 86,210 L204,146 C220,137 220,119 204,110 Z" />
        <path android:pathData="M0,0 H104 L78,256 H0 Z M118,0 H148 L122,256 H92 Z M162,0 H256 V256 H136 Z">
            <aapt:attr name="android:fillColor">
                <gradient
                    android:type="linear"
                    android:startX="52"
                    android:startY="60"
                    android:endX="216"
                    android:endY="200"
                    android:startColor="${argb(icon.from)}"
                    android:endColor="${argb(icon.to)}" />
            </aapt:attr>
        </path>
    </group>
</vector>
`;

const adaptive = (id) => `<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@drawable/mobile_icon_background" />
    <foreground>
        <inset android:drawable="@drawable/fluxa_logo_${id}" android:inset="18dp" />
    </foreground>
    <monochrome>
        <inset android:drawable="@drawable/fluxa_logo_${id}" android:inset="18dp" />
    </monochrome>
</adaptive-icon>
`;

const alias = (icon) => {
  const isDefault = icon.id === defaultId;
  const res = isDefault ? "@mipmap/ic_launcher" : `@mipmap/ic_launcher_${icon.id}`;
  return `        <activity-alias
            android:name=".ui.icon.${icon.id}"
            android:targetActivity=".ui.MainActivity"
            android:enabled="${isDefault}"
            android:exported="true"
            android:icon="${res}"
            android:roundIcon="${res}">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity-alias>`;
};

const writeAndroid = () => {
  for (const icon of icons) {
    write(join(android, `main/res/drawable/fluxa_logo_${icon.id}.xml`), vector(icon));
    if (icon.id !== defaultId) {
      write(join(android, `mobile/res/mipmap-anydpi-v26/ic_launcher_${icon.id}.xml`), adaptive(icon.id));
    }
  }
  const manifest = join(android, "mobile/AndroidManifest.xml");
  const text = readFileSync(manifest, "utf8");
  const start = "        <!-- app-icons:start -->";
  const end = "        <!-- app-icons:end -->";
  const block = [start, ...icons.map(alias), end].join("\n");
  const pattern = new RegExp(`${start.trim()}[\\s\\S]*${end.trim()}`);
  if (!pattern.test(text)) throw new Error("mobile manifest is missing the app-icons markers");
  writeFileSync(manifest, text.replace(new RegExp(`\\s*${start.trim()}[\\s\\S]*${end.trim()}`), `\n${block}`));
};

const appIconSet = (dir, icon) => {
  const images = [
    { file: "light.png", svg: tileSvg(icon, 1024, "#131313") },
    { file: "dark.png", svg: tileSvg(icon, 1024, null), appearance: "dark" },
    { file: "tinted.png", svg: tileSvg(icon, 1024, null), appearance: "tinted", gray: true },
  ];
  for (const image of images) png(image.svg, join(dir, image.file), { gray: image.gray });
  write(join(dir, "Contents.json"), JSON.stringify({
    images: images.map((image) => ({
      ...(image.appearance ? { appearances: [{ appearance: "luminosity", value: image.appearance }] } : {}),
      filename: image.file,
      idiom: "universal",
      platform: "ios",
      size: "1024x1024",
    })),
    info: { author: "xcode", version: 1 },
  }, null, 2) + "\n");
};

const layer = (dir, name, svg) => {
  png(svg, join(dir, `${name}.imagestacklayer/Content.imageset/${name}.png`));
  write(join(dir, `${name}.imagestacklayer/Content.imageset/Contents.json`), JSON.stringify({
    images: [{ filename: `${name}.png`, idiom: "tv" }],
    info: { author: "xcode", version: 1 },
  }, null, 2) + "\n");
  write(join(dir, `${name}.imagestacklayer/Contents.json`), JSON.stringify({ info: { author: "xcode", version: 1 } }, null, 2) + "\n");
};

const imageStack = (dir, icon, width, height) => {
  const scale = Math.min(width, height) * 0.7 / 256;
  const inner = markSvg(icon).replace(/^<svg[^>]*>/, "").replace(/<\/svg>$/, "");
  const mark = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><g transform="translate(${(width - 256 * scale) / 2} ${(height - 256 * scale) / 2}) scale(${scale})">${inner}</g></svg>`;
  layer(dir, "Back", `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><rect width="100%" height="100%" fill="#131313"/></svg>`);
  layer(dir, "Front", mark);
  write(join(dir, "Contents.json"), JSON.stringify({
    info: { author: "xcode", version: 1 },
    layers: [{ filename: "Front.imagestacklayer" }, { filename: "Back.imagestacklayer" }],
  }, null, 2) + "\n");
};

const imageSet = (dir, name, svg, idiom) => {
  png(svg, join(dir, `${name}.png`));
  write(join(dir, "Contents.json"), JSON.stringify({
    images: [{ filename: `${name}.png`, idiom }],
    info: { author: "xcode", version: 1 },
  }, null, 2) + "\n");
};

const writeApple = () => {
  rmSync(xcassets, { recursive: true, force: true });
  write(join(xcassets, "Contents.json"), JSON.stringify({ info: { author: "xcode", version: 1 } }, null, 2) + "\n");
  for (const icon of icons) {
    appIconSet(join(xcassets, icon.id === defaultId ? "AppIcon.appiconset" : `AppIcon-${icon.id}.appiconset`), icon);
  }
  const main = icons.find((icon) => icon.id === defaultId);
  const brandDir = join(xcassets, "TvAppIcon.brandassets");
  imageStack(join(brandDir, "App Icon.imagestack"), main, 400, 240);
  imageStack(join(brandDir, "App Icon - App Store.imagestack"), main, 1280, 768);
  const shelf = (w, h) => {
    const s = h * 0.6 / 256;
    const inner = markSvg(main).replace(/^<svg[^>]*>/, "").replace(/<\/svg>$/, "");
    return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}"><rect width="100%" height="100%" fill="#131313"/><g transform="translate(${(w - 256 * s) / 2} ${(h - 256 * s) / 2}) scale(${s})">${inner}</g></svg>`;
  };
  imageSet(join(brandDir, "Top Shelf Image.imageset"), "shelf", shelf(1920, 720), "tv");
  imageSet(join(brandDir, "Top Shelf Image Wide.imageset"), "shelf-wide", shelf(2320, 720), "tv");
  write(join(brandDir, "Contents.json"), JSON.stringify({
    assets: [
      { filename: "App Icon - App Store.imagestack", idiom: "tv", role: "primary-app-icon", size: "1280x768" },
      { filename: "App Icon.imagestack", idiom: "tv", role: "primary-app-icon", size: "400x240" },
      { filename: "Top Shelf Image Wide.imageset", idiom: "tv", role: "top-shelf-image-wide", size: "2320x720" },
      { filename: "Top Shelf Image.imageset", idiom: "tv", role: "top-shelf-image", size: "1920x720" },
    ],
    info: { author: "xcode", version: 1 },
  }, null, 2) + "\n");
};

const writeWeb = () => {
  write(join(root, "native/fluxa-web/www/favicon.svg"), markSvg(icons.find((icon) => icon.id === defaultId)));
};

if (!existsSync(join(android, "mobile/AndroidManifest.xml"))) throw new Error("run from the repository");
writeAndroid();
writeApple();
writeWeb();
