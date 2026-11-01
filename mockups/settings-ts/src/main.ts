import { categories, shortcuts, type Category, type Def, type Group, type Section } from "./data";

type Value = string | number | boolean;

const storeKey = "fluxa-settings-ts";
const defs = new Map<string, { def: Def; cat: Category; sec: Section }>();
for (const cat of categories)
  for (const sec of cat.sections)
    for (const group of sec.groups)
      for (const def of group.items) defs.set(def.key, { def, cat, sec });

const kids = new Map<string, Def[]>();
for (const { def } of defs.values()) if (def.needs) kids.set(def.needs, [...(kids.get(def.needs) ?? []), def]);
const opened: Record<string, boolean> = {};

const values: Record<string, Value> = {};
for (const [key, { def }] of defs) values[key] = def.def;
try {
  Object.assign(values, JSON.parse(localStorage.getItem(storeKey) ?? "{}"));
} catch {}

const icons: Record<string, string> = {
  user: '<circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/>',
  palette: '<path d="M12 3a9 9 0 1 0 0 18c1.2 0 2-.8 2-1.8 0-.5-.2-.9-.5-1.3-.3-.4-.5-.8-.5-1.3 0-1 .8-1.8 1.8-1.8H17a4 4 0 0 0 4-4C21 6.6 17 3 12 3Z"/><circle cx="7.5" cy="11" r="1"/><circle cx="10" cy="7" r="1"/><circle cx="15" cy="7" r="1"/>',
  play: '<path d="M7 4.5v15l12-7.5-12-7.5Z"/>',
  package: '<path d="m12 3 8 4.5v9L12 21l-8-4.5v-9L12 3Z"/><path d="m4 7.5 8 4.5 8-4.5M12 12v9"/>',
  download: '<path d="M12 4v11m0 0-4-4m4 4 4-4M5 20h14"/>',
  keyboard: '<rect x="2.5" y="6" width="19" height="12" rx="2.5"/><path d="M6.5 10h.01M10 10h.01M13.5 10h.01M17 10h.01M7.5 14h9"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  back: '<path d="m15 5-7 7 7 7"/>',
  chevron: '<path d="m9 5 7 7-7 7"/>',
  reset: '<path d="M4 12a8 8 0 1 0 3-6.3M4 4v4h4"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  close: '<path d="m6 6 12 12M18 6 6 18"/>',
  film: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M7 3v18M17 3v18M3 8h4M3 12h4M3 16h4M17 8h4M17 12h4M17 16h4"/>',
  home: '<path d="m3 11 9-8 9 8M5 10v10h14V10"/>',
  layout: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/>',
  image: '<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-5-5L5 21"/>',
  moon: '<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z"/>',
  captions: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 11h3M14 11h3M7 15h6"/>',
  sliders: '<path d="M4 6h8M18 6h2M4 12h2M12 12h8M4 18h10M20 18h0"/><circle cx="15" cy="6" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="18" r="2"/>',
  scan: '<path d="M4 8V5a1 1 0 0 1 1-1h3M16 4h3a1 1 0 0 1 1 1v3M20 16v3a1 1 0 0 1-1 1h-3M8 20H5a1 1 0 0 1-1-1v-3M4 12h16"/>',
  badge: '<path d="m12 3 2.5 2 3.2.2.2 3.2 2 2.5-2 2.5-.2 3.2-3.2.2L12 21l-2.5-2-3.2-.2-.2-3.2-2-2.5 2-2.5.2-3.2 3.2-.2L12 3Z"/><path d="m9 12 2 2 4-4"/>',
};

const svgNs = "http://www.w3.org/2000/svg";

function icon(name: string, size = 20): SVGElement {
  const svg = document.createElementNS(svgNs, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "1.8");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.setAttribute("aria-hidden", "true");
  svg.innerHTML = icons[name] ?? "";
  return svg;
}

type Child = Node | string | null | false | undefined;

function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Record<string, unknown> = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [name, value] of Object.entries(props)) {
    if (value === undefined || value === false || value === null) continue;
    if (name === "class") el.className = String(value);
    else if (name === "style") el.setAttribute("style", String(value));
    else if (name.startsWith("on")) el.addEventListener(name.slice(2).toLowerCase(), value as EventListener);
    else if (value === true) el.setAttribute(name, "");
    else el.setAttribute(name, String(value));
  }
  for (const child of children) {
    if (child === null || child === false || child === undefined) continue;
    el.append(child);
  }
  return el;
}

const labelNames: Record<string, string> = {
  mpv: "Built-in player",
  external: "External player",
  imdb: "IMDb",
  mdblist: "MDBList",
  xsmall: "XS",
  small: "S",
  medium: "M",
  large: "L",
  xlarge: "XL",
  pill: "Extra rounded",
  sidebar: "Sidebar",
  topbar: "Top bar",
  hover: "On hover",
  expanded: "Always expanded",
  collapsed: "Always collapsed",
  inherit: "Match global",
  ultra_fast: "Ultra fast",
  anime4k_s: "Fast",
  anime4k_m: "Balanced",
  anime4k_l: "Quality",
  a: "Mode A",
  b: "Mode B",
  c: "Mode C",
  "2gb": "2 GB",
  "5gb": "5 GB",
  "10gb": "10 GB",
  "-1": "Unlimited",
  "0": "Off",
  default: "Default",
  device: "Device language",
  original: "Original language",
  forced: "Forced only",
  none: "None",
  no: "Off",
  yes: "On",
  sdr: "Force SDR",
  base_layer: "Base layer",
  regex: "Regex match",
  first: "First available",
  best: "Best match",
  manual: "Manual",
  async: "In background",
  pregenerate: "Ahead of time",
  display_resample: "Display resample",
  time: "Time left",
  percent: "Percent",
  remaining: "Remaining",
  watched: "Watched",
  posters: "Posters",
  chips: "Chips",
  dropdown: "Dropdown",
};

let languageNames: Intl.DisplayNames | null = null;
try {
  languageNames = new Intl.DisplayNames(["en"], { type: "language" });
} catch {}

function optionLabel(def: Def, option: string): string {
  const base = labelNames[option] ?? languageLabel(def, option) ?? humanize(option);
  if (!def.unit || option in labelNames) return base;
  return def.unit.length > 1 && /^[a-z]/i.test(def.unit) ? `${base} ${def.unit}` : `${base}${def.unit}`;
}

function languageLabel(def: Def, option: string): string | null {
  if (!def.key.toLowerCase().includes("language") && def.key !== "language") return null;
  if (!/^[a-z]{2}$/.test(option)) return null;
  try {
    return languageNames?.of(option) ?? null;
  } catch {
    return null;
  }
}

function humanize(text: string): string {
  const spaced = text.replace(/_/g, " ");
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

function isOn(key: string): boolean {
  const v = values[key];
  if (typeof v === "boolean") return v;
  return v !== "off" && v !== "none" && v !== "";
}

function save() {
  try {
    localStorage.setItem(storeKey, JSON.stringify(values));
  } catch {}
}

function set(key: string, value: Value) {
  values[key] = value;
  save();
  applyTheme();
  syncRows();
  refreshPreviews();
}

function isModified(def: Def): boolean {
  return String(values[def.key]) !== String(def.def);
}

function luminance(hex: string): number {
  const n = parseInt(hex.slice(1), 16);
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  return (0.299 * r + 0.587 * g + 0.114 * b) / 255;
}

function applyTheme() {
  const root = document.documentElement;
  const picked = String(values.accentColorArgb);
  const light = matchMedia("(prefers-color-scheme: light)").matches;
  const accent = picked.toUpperCase() === "#FFFFFF" && light ? "#1A1A1E" : picked;
  root.style.setProperty("--accent", accent);
  root.style.setProperty("--on-accent", luminance(accent) > 0.6 ? "#0b0b0f" : "#ffffff");
  root.style.fontSize = `${(16 * Number(values.uiScale)) / 100}px`;
  root.dataset.density = String(values.interfaceDensity);
  root.dataset.amoled = String(values.amoledMode);
}

function toggleControl(def: Def): HTMLElement {
  const button = h("button", {
    class: "switch",
    role: "switch",
    type: "button",
    "aria-label": def.label,
    "aria-checked": String(values[def.key]),
    "data-key": def.key,
    onClick: () => set(def.key, !values[def.key]),
  }, h("span", { class: "knob" }));
  return button;
}

function isRadio(def: Def): boolean {
  return def.kind === "choice" && !visualKeys.has(def.key) && (def.options?.length ?? 0) <= 5;
}

function isOpen(def: Def): boolean {
  return opened[def.key] ?? (def.kind === "toggle" && values[def.key] === true);
}

function toggleOpen(def: Def) {
  opened[def.key] = !isOpen(def);
  syncRows();
}

function chevronButton(def: Def): HTMLElement {
  return h("button", {
    type: "button",
    class: "xchev",
    "aria-label": `Show options for ${def.label}`,
    onClick: (e: Event) => {
      e.stopPropagation();
      toggleOpen(def);
    },
  }, icon("chevron", 16));
}

function choiceControl(def: Def, flat = false): HTMLElement {
  if (!flat && isRadio(def))
    return h("div", { class: "valbox" }, h("span", { class: "valtext", "data-valtext": def.key }, optionLabel(def, String(values[def.key]))), chevronButton(def));
  const select = h("select", {
    class: "select",
    "aria-label": def.label,
    "data-key": def.key,
    onChange: (e: Event) => set(def.key, (e.target as HTMLSelectElement).value),
  });
  for (const option of def.options ?? []) {
    const el = h("option", { value: option }, optionLabel(def, option));
    if (String(values[def.key]) === option) el.selected = true;
    select.append(el);
  }
  return h("div", { class: "select-wrap" }, select, icon("chevron", 14));
}

function radioList(def: Def): HTMLElement {
  const group = h("div", { class: "radios", role: "radiogroup", "aria-label": def.label, "data-rad": def.key });
  for (const option of def.options ?? []) {
    group.append(h("button", {
      type: "button",
      class: "radio",
      role: "radio",
      "data-value": option,
      "aria-checked": String(String(values[def.key]) === option),
      onClick: () => set(def.key, option),
    }, h("i", {}), h("span", {}, optionLabel(def, option))));
  }
  return group;
}

function colorControl(def: Def): HTMLElement {
  const group = h("div", { class: "swatches", role: "radiogroup", "aria-label": def.label, "data-key": def.key });
  for (const option of def.options ?? []) {
    group.append(h("button", {
      type: "button",
      class: "swatch",
      role: "radio",
      style: `--c:${option}`,
      title: option,
      "aria-label": option,
      "data-value": option,
      "aria-checked": String(String(values[def.key]).toLowerCase() === option.toLowerCase()),
      onClick: () => set(def.key, option),
    }, icon("check", 14)));
  }
  return group;
}

function rangeControl(def: Def): HTMLElement {
  const out = h("output", { class: "range-value" });
  const input = h("input", {
    type: "range",
    class: "range",
    min: def.min,
    max: def.max,
    step: def.step,
    value: values[def.key],
    "aria-label": def.label,
    "data-key": def.key,
    onInput: (e: Event) => {
      const v = Number((e.target as HTMLInputElement).value);
      out.textContent = `${v}${def.unit ?? ""}`;
      paintRange(e.target as HTMLInputElement);
      set(def.key, v);
    },
  });
  out.textContent = `${values[def.key]}${def.unit ?? ""}`;
  paintRange(input);
  return h("div", { class: "range-wrap" }, input, out);
}

function paintRange(input: HTMLInputElement) {
  const min = Number(input.min);
  const max = Number(input.max);
  input.style.setProperty("--fill", `${((Number(input.value) - min) / (max - min)) * 100}%`);
}

const sectionIcon: Record<string, string> = {
  account: "user", tmdb: "film", theme: "palette", home: "home", details: "layout", posters: "image", ambient: "moon",
  playback: "play", subtitles: "captions", player: "sliders", previews: "scan", badges: "badge", content: "package",
  downloads: "download", shortcuts: "keyboard", about: "info",
};

const tileColor: Record<string, [string, string]> = {
  account: ["#7c7cff", "#5a4de6"], appearance: ["#ff6fae", "#f2742e"], playback: ["#37d67a", "#119c8c"],
  content: ["#ffb53d", "#ee5a3c"], downloads: ["#4aa8ff", "#1cc4d6"], shortcuts: ["#9aa3b5", "#667085"], about: ["#a3a3ad", "#6e6e78"],
};

function tile(_cat: string, name: string, size = 1.4): HTMLElement {
  return h("span", { class: "tile" }, icon(name, Math.round(size * 12)));
}

const iconGradients: Record<string, [string, string]> = {
  ember: ["#ff7a3d", "#e5230f"], sunset: ["#ffb347", "#d1337a"], ocean: ["#3bb2ff", "#1c4fd8"], dusk: ["#8a63ff", "#2b1d73"],
  lagoon: ["#2de2c3", "#0b6f9a"], ruby: ["#ff4d6d", "#7a0c2e"], citrus: ["#ffe14d", "#8fd400"], peach: ["#ffc2a3", "#ff6f91"],
  lime: ["#b8ff4d", "#1fa84f"], aurora: ["#6dffb0", "#7a4dff"], neon: ["#ff3df2", "#3d8bff"], flare: ["#ffdf6b", "#ff4a1c"],
  ice: ["#d8f3ff", "#58a6ff"], rose: ["#ffb3d1", "#d6336c"], silver: ["#f1f1f4", "#8b8b96"], gold: ["#ffe08a", "#c98a0b"],
  jade: ["#7dffc7", "#0b7a55"], blaze: ["#ffd24d", "#ff2e2e"], orange: ["#ffa24d", "#e8590c"],
};

const badgeThemes: Record<string, string> = {
  vivid: "#ff4d4d", classic: "#d9d9df", soft: "#a8c7ff", neon: "#3dffb5", gold: "#f5c542", sunset: "#ff8a4d", cinema: "#c7a0ff",
};

function thumb(key: string, option: string): HTMLElement | null {
  if (key === "cardCornerPreset") return h("i", { class: "th-card", style: `border-radius:${radius[option]}` });
  if (key === "cardLayout") return h("i", { class: `th-card ${option === "horizontal" ? "wide" : ""}` });
  if (key === "posterWidthPreset") return h("i", { class: "th-card", style: `width:${{ xsmall: 16, small: 22, medium: 28, large: 36, xlarge: 44 }[option] ?? 28}px` });
  if (key === "navLayout") return h("i", { class: `th-nav ${option}` }, h("b", {}), h("u", {}));
  if (key === "interfaceDensity") {
    const gap = { small: 3, medium: 6, large: 10 }[option] ?? 6;
    return h("i", { class: "th-lines", style: `gap:${gap}px` }, h("b", {}), h("b", {}), h("b", {}));
  }
  if (key === "homeHeroArtwork") return h("i", { class: `th-hero ${option}` });
  if (key === "posterRatingPosition") return h("i", { class: `th-card pos-${option}` }, h("b", {}));
  if (key === "appIcon") {
    const [a, b] = iconGradients[option] ?? ["#888", "#444"];
    return h("i", { class: "th-app", style: `background:linear-gradient(145deg,${a},${b})` });
  }
  if (key === "streamBadgeTheme") return h("i", { class: "th-badge", style: `--c:${badgeThemes[option] ?? "#fff"}` });
  return null;
}

function visualControl(def: Def): HTMLElement {
  const group = h("div", { class: "vgrid", role: "radiogroup", "aria-label": def.label, "data-key": def.key });
  for (const option of def.options ?? []) {
    group.append(h("button", {
      type: "button",
      class: "vopt",
      role: "radio",
      "data-value": option,
      "aria-checked": String(String(values[def.key]) === option),
      onClick: () => set(def.key, option),
    }, h("span", { class: "vpic" }, thumb(def.key, option)), h("span", { class: "vname" }, optionLabel(def, option))));
  }
  return group;
}

const visualKeys = new Set(["appIcon"]);

function control(def: Def, flat = false): HTMLElement {
  if (def.kind === "choice" && visualKeys.has(def.key)) return visualControl(def);
  if (def.kind === "toggle") return toggleControl(def);
  if (def.kind === "color") return colorControl(def);
  if (def.kind === "range") return rangeControl(def);
  return choiceControl(def, flat);
}

function plainRow(def: Def, flat: boolean, expandable: boolean): HTMLElement {
  const ctl = control(def, flat);
  const controlBox = h("div", { class: "row-control" }, ctl);
  if (expandable && !isRadio(def)) controlBox.append(chevronButton(def));
  const el = h("div", { class: `row${def.kind === "choice" && visualKeys.has(def.key) ? " stack" : ""}${expandable ? " expandable" : ""}`, "data-row": def.key },
    h("div", { class: "row-text" },
      h("div", { class: "row-label" }, def.label, h("i", { class: "dot", title: "Changed from default" })),
      def.desc && h("div", { class: "row-desc" }, def.desc),
    ),
    controlBox,
  );
  if (expandable && isRadio(def)) el.addEventListener("click", () => toggleOpen(def));
  return el;
}

function row(def: Def, nested = true): HTMLElement {
  const sub = nested ? kids.get(def.key) ?? [] : [];
  const radio = nested && isRadio(def);
  if (!sub.length && !radio) return plainRow(def, !nested, false);
  const body = h("div", { class: "xbody" },
    radio && radioList(def),
    ...sub.map((k) => row(k)),
  );
  return h("div", { class: `xwrap${isOpen(def) ? " open" : ""}`, "data-x": def.key }, plainRow(def, false, true), body);
}

function syncRows() {
  for (const el of document.querySelectorAll<HTMLElement>("[data-x]")) {
    const entry = defs.get(el.dataset.x!);
    if (entry) el.classList.toggle("open", isOpen(entry.def));
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-valtext]")) {
    const entry = defs.get(el.dataset.valtext!);
    if (entry) el.textContent = optionLabel(entry.def, String(values[entry.def.key]));
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-rad]")) {
    const value = String(values[el.dataset.rad!]);
    for (const btn of el.querySelectorAll<HTMLElement>("[data-value]")) btn.setAttribute("aria-checked", String(btn.dataset.value === value));
  }
  for (const [key, { def }] of defs) {
    for (const el of document.querySelectorAll<HTMLElement>(`[data-row="${key}"]`)) {
      el.classList.toggle("modified", isModified(def));
      el.classList.toggle("disabled", !!def.needs && !isOn(def.needs));
      el.querySelectorAll("input,select,button").forEach((c) => {
        (c as HTMLButtonElement).disabled = !!def.needs && !isOn(def.needs);
      });
      for (const sw of el.querySelectorAll<HTMLElement>(".switch")) sw.setAttribute("aria-checked", String(values[key]));
      for (const sel of el.querySelectorAll<HTMLSelectElement>("select")) sel.value = String(values[key]);
      for (const btn of el.querySelectorAll<HTMLElement>("[data-value]"))
        btn.setAttribute("aria-checked", String(String(values[key]).toLowerCase() === btn.dataset.value!.toLowerCase()));
    }
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-group]")) {
    const any = el.querySelector(".row.modified");
    el.classList.toggle("has-changes", !!any);
  }
}

function resetGroup(group: Group) {
  for (const def of group.items) values[def.key] = def.def;
  save();
  applyTheme();
  const page = document.getElementById("page")!;
  const keep = page.scrollTop;
  render();
  page.scrollTop = keep;
}

function groupCard(group: Group): HTMLElement {
  return h("section", { class: "group", "data-group": group.title },
    h("div", { class: "group-head" },
      h("h2", {}, group.title),
      h("button", { class: "link", type: "button", onClick: () => resetGroup(group) }, icon("reset", 14), "Reset to defaults"),
    ),
    h("div", { class: "card" }, ...group.items.filter((d) => !d.needs || !defs.has(d.needs)).map((d) => row(d))),
  );
}

const radius: Record<string, string> = { sharp: "2px", classic: "6px", soft: "10px", rounded: "16px", pill: "26px" };
const posterWidth: Record<string, number> = { xsmall: 76, small: 92, medium: 112, large: 136, xlarge: 160 };

function posterPreview(): HTMLElement {
  const horizontal = values.cardLayout === "horizontal" || values.posterLandscapeMode === true;
  const w = (posterWidth[String(values.posterWidthPreset)] ?? 112) * (horizontal ? 1.55 : 1);
  const h2 = horizontal ? w * 0.58 : w * 1.5;
  const rating = values.posterRatingBadge === true;
  const pos = String(values.posterRatingPosition);
  const corner = values.posterOverlaysEnabled === true;
  const card = (n: number, art: string, progress: number) => {
    const el = h("div", { class: "pv-poster", style: `width:${w}px;height:${h2}px;border-radius:${radius[String(values.cardCornerPreset)]};background:${art}` });
    if (values.posterLift === true) el.classList.add("lift");
    if (corner && rating) el.append(h("span", { class: `pv-badge pv-${pos}` }, n === 0 ? "8.4" : n === 1 ? "7.9" : "9.1"));
    if (corner && values.posterSavedBadge === true && n === 0) el.append(h("span", { class: "pv-saved" }, icon("check", 12)));
    if (corner && values.posterWatchedBadge === true && n === 2) el.append(h("span", { class: "pv-saved" }, icon("check", 12)));
    if (corner && values.posterStatusBadge === true && n === 1) el.append(h("span", { class: "pv-sash" }, "New"));
    if (corner && values.posterProgressBar === true && progress > 0)
      el.append(h("span", { class: "pv-progress" }, h("i", { style: `width:${progress}%` })));
    if (values.posterTopNumbers === true && values.posterTopNumbersPosition === "overlay")
      el.append(h("span", { class: "pv-num" }, String(n + 1)));
    return h("figure", { class: "pv-item" },
      values.posterTopNumbers === true && values.posterTopNumbersPosition === "beside" && h("span", { class: "pv-num beside" }, String(n + 1)),
      el,
      values.posterHideTitles !== true && h("figcaption", {}, ["Midnight Orbit", "Paper Moons", "Salt & Ember"][n]),
    );
  };
  return h("div", { class: "preview poster-pv", "aria-label": "Live preview" },
    h("span", { class: "preview-tag" }, "Live preview"),
    h("div", { class: "pv-row" },
      card(0, "linear-gradient(160deg,#3b4cca,#1b1f4d 60%,#0e1030)", 45),
      card(1, "linear-gradient(160deg,#e0683c,#7a2430 60%,#2a0e18)", 0),
      card(2, "linear-gradient(160deg,#2f9d78,#124a45 60%,#08201f)", 100),
    ),
  );
}

function subsPreview(): HTMLElement {
  const hex = (color: string, a: string) => {
    const n = parseInt(color.slice(1), 16);
    return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${a})`;
  };
  const size = Number(values.subtitleSize);
  const outline = Number(values.subtitleOutlineSize);
  const oc = hex(String(values.subtitleOutlineColor), String(values.subtitleOutlineOpacity));
  const shadows: string[] = [];
  if (outline > 0)
    for (let a = 0; a < 8; a++) {
      const ang = (a / 8) * Math.PI * 2;
      shadows.push(`${(Math.cos(ang) * outline).toFixed(1)}px ${(Math.sin(ang) * outline).toFixed(1)}px 0 ${oc}`);
    }
  if (values.subtitleShadow === true) shadows.push("0 2px 6px rgba(0,0,0,.8)");
  const bottom = 100 - Number(values.subtitlePosition) + 8;
  return h("div", { class: "preview subs-pv", "aria-label": "Live preview" },
    h("span", { class: "preview-tag" }, "Live preview"),
    h("div", { class: "frame" },
      h("div", { class: "frame-sky" }),
      h("div", {
        class: "sub",
        style: `bottom:${bottom}%;font-size:${(size / 100) * 1.1}rem;font-weight:${values.subtitleBold === true ? 700 : 500};color:${hex(String(values.subtitleColor), String(values.subtitleTextOpacity))};background:${hex(String(values.subtitleBackgroundColor), String(values.subtitleBackgroundOpacity))};text-shadow:${shadows.join(",") || "none"}`,
      }, "Nobody leaves the island\nuntil the tide turns."),
    ),
  );
}

function themePreview(): HTMLElement {
  const top = values.navLayout === "topbar";
  const labels = values.navLabels === true;
  const floating = values.navFloating === true;
  const names = ["Home", "Library", "Discover", "Calendar"];
  const nav = h("div", { class: `mock-nav${top ? " top" : ""}${floating ? " floating" : ""}` },
    ...names.map((n, i) => h("span", { class: `mock-item${i === 0 ? " on" : ""}` }, h("i", {}), labels && h("b", {}, n))),
  );
  const body = h("div", { class: "mock-body" },
    h("div", { class: "mock-hero" }, h("span", { class: "mock-title" }), h("span", { class: "mock-meta" }), h("span", { class: "mock-play" })),
    h("div", { class: "mock-cards" }, ...[0, 1, 2, 3, 4].map(() => h("i", { style: `border-radius:${radius[String(values.cardCornerPreset)]}` }))),
  );
  return h("div", { class: "preview theme-pv", "aria-label": "Live preview" },
    h("span", { class: "preview-tag" }, "Live preview"),
    h("div", { class: `mock${top ? " col" : ""}` }, nav, body),
  );
}

function previewFor(sec: Section): HTMLElement | null {
  if (sec.preview === "poster") return posterPreview();
  if (sec.preview === "subs") return subsPreview();
  if (sec.preview === "theme") return themePreview();
  return null;
}

function refreshPreviews() {
  for (const host of document.querySelectorAll<HTMLElement>("[data-preview]")) {
    const sec = allSections.find((x) => x.sec.id === host.dataset.preview)?.sec;
    const next = sec && previewFor(sec);
    if (next) host.replaceChildren(...next.childNodes);
  }
}

function sectionBlock(sec: Section): HTMLElement {
  const pv = previewFor(sec);
  const host = pv && h("div", { class: `preview ${[...pv.classList].filter((c) => c !== "preview").join(" ")}`, "data-preview": sec.id }, ...pv.childNodes);
  return h("div", { class: "sec" }, host, ...sec.groups.map(groupCard));
}

function extras(sec: Section): HTMLElement | null {
  if (sec.id === "shortcuts")
    return h("section", { class: "group" },
      h("div", { class: "group-head" }, h("h2", {}, "Reference")),
      h("div", { class: "card" }, ...shortcuts.map(([name, keys]) =>
        h("div", { class: "row" }, h("div", { class: "row-text" }, h("div", { class: "row-label" }, name)),
          h("div", { class: "row-control" }, h("span", { class: "keys" }, ...keys.split(" ").map((k) => h("kbd", {}, k)))),
        ))),
    );
  if (sec.id === "about")
    return h("section", { class: "group" },
      h("div", { class: "about" },
        h("div", { class: "about-mark" }, "F"),
        h("div", {}, h("div", { class: "about-name" }, "Fluxa"), h("div", { class: "about-ver" }, "Version 0.1.30")),
        h("button", { class: "btn primary", type: "button" }, "Check for updates"),
      ),
      h("div", { class: "card" },
        ...["Open source licenses", "Privacy policy", "Report a problem"].map((label) =>
          h("button", { class: "row nav-row", type: "button" }, h("div", { class: "row-text" }, h("div", { class: "row-label" }, label)), icon("chevron", 16))),
      ),
    );
  return null;
}

const allSections = categories.flatMap((cat) => cat.sections.map((sec) => ({ cat, sec })));
let activeSec = location.hash.slice(1);
if (!allSections.some((x) => x.sec.id === activeSec)) activeSec = "account";
let query = "";

function renderSide() {
  const nav = document.getElementById("rail-list")!;
  nav.replaceChildren(...categories.map((cat) =>
    h("div", { class: "side-group" },
      h("div", { class: "side-label" }, cat.title),
      ...cat.sections.map((sec) => h("button", {
        class: "side-item",
        type: "button",
        "aria-current": String(sec.id === activeSec && !query),
        onClick: () => go(sec.id),
      }, tile(cat.id, sectionIcon[sec.id] ?? cat.icon), h("span", {}, sec.title), icon("chevron", 14))),
    )));
}

function go(id: string) {
  activeSec = id;
  query = "";
  (document.getElementById("search") as HTMLInputElement).value = "";
  history.replaceState(null, "", `#${id}`);
  document.body.dataset.view = "page";
  render();
  document.getElementById("page")!.scrollTop = 0;
}

function renderResults(): HTMLElement {
  const q = query.toLowerCase();
  const hits = allSections.flatMap(({ cat, sec }) =>
    sec.groups.flatMap((g) => g.items.filter((def) =>
      `${def.label} ${def.desc ?? ""} ${sec.title} ${cat.title} ${def.key}`.toLowerCase().includes(q)).map((def) => ({ cat, sec, def }))));
  const wrap = h("div", { class: "results" });
  wrap.append(h("div", { class: "page-head" }, h("h1", {}, "Search results"), h("p", {}, `${hits.length} ${hits.length === 1 ? "setting" : "settings"} matching “${query}”`)));
  if (!hits.length) {
    wrap.append(h("div", { class: "empty" }, icon("search", 28), h("b", {}, "No settings found"), h("span", {}, "Check the spelling or try a different word.")));
    return wrap;
  }
  const bySection = new Map<string, typeof hits>();
  for (const hit of hits) bySection.set(hit.sec.id, [...(bySection.get(hit.sec.id) ?? []), hit]);
  for (const list of bySection.values()) {
    const { cat, sec } = list[0];
    wrap.append(h("section", { class: "group" },
      h("div", { class: "group-head" },
        h("h2", {}, `${cat.title} › ${sec.title}`),
        h("button", { class: "link show", type: "button", onClick: () => go(sec.id) }, "Open", icon("chevron", 14)),
      ),
      h("div", { class: "card" }, ...list.map(({ def }) => row(def, false))),
    ));
  }
  return wrap;
}

function render() {
  renderSide();
  const page = document.getElementById("page")!;
  const title = document.getElementById("bar-title")!;
  if (query) {
    title.textContent = "Search";
    page.replaceChildren(renderResults());
    syncRows();
    return;
  }
  const { cat, sec } = allSections.find((x) => x.sec.id === activeSec)!;
  title.textContent = sec.title;
  const extra = extras(sec);
  page.replaceChildren(
    h("div", { class: "page-head" },
      h("h1", {}, sec.title),
      h("p", {}, sec.desc),
    ),
    sectionBlock(sec),
    ...(extra ? [extra] : []),
  );
  syncRows();
}

document.getElementById("search")!.addEventListener("input", (e) => {
  query = (e.target as HTMLInputElement).value.trim();
  document.body.dataset.view = "page";
  render();
});

document.getElementById("back")!.addEventListener("click", () => {
  query = "";
  (document.getElementById("search") as HTMLInputElement).value = "";
  document.body.dataset.view = "list";
});

window.addEventListener("keydown", (e) => {
  const search = document.getElementById("search") as HTMLInputElement;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    search.focus();
    search.select();
  } else if (e.key === "Escape" && document.activeElement === search) {
    search.value = "";
    query = "";
    search.blur();
    render();
  }
});

applyTheme();
document.body.dataset.view = location.hash ? "page" : "list";
render();
