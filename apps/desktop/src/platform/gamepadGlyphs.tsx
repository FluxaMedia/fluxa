import { assetUrl } from './assets';

export type GamepadBrand = 'xbox' | 'playstation' | 'nintendo' | 'generic';

export function detectGamepadBrand(id: string): GamepadBrand {
  const lower = id.toLowerCase();
  if (lower.includes('xbox') || lower.includes('xinput')) return 'xbox';
  if (
    lower.includes('054c') ||
    lower.includes('dualshock') ||
    lower.includes('dualsense') ||
    lower.includes('sony') ||
    lower.includes('playstation')
  )
    return 'playstation';
  if (lower.includes('057e') || lower.includes('nintendo') || lower.includes('switch') || lower.includes('joy-con')) return 'nintendo';
  return 'generic';
}

const XBOX_LABELS: Record<number, string> = { 0: 'A', 1: 'B', 2: 'X', 3: 'Y' };
const PLAYSTATION_LABELS: Record<number, string> = { 0: 'Cross', 1: 'Circle', 2: 'Square', 3: 'Triangle' };
const NINTENDO_LABELS: Record<number, string> = { 0: 'B', 1: 'A', 2: 'Y', 3: 'X' };
const DPAD_LABELS: Record<number, string> = { 12: '↑', 13: '↓', 14: '←', 15: '→' };

const TEXT_LABELS: Record<GamepadBrand, Record<number, string>> = {
  xbox: { 4: 'LB', 5: 'RB', 6: 'LT', 7: 'RT', 8: 'View', 9: 'Menu', 10: 'LS', 11: 'RS', 16: 'Guide' },
  playstation: { 4: 'L1', 5: 'R1', 6: 'L2', 7: 'R2', 8: 'Share', 9: 'Options', 10: 'L3', 11: 'R3', 16: 'PS' },
  nintendo: { 4: 'L', 5: 'R', 6: 'ZL', 7: 'ZR', 8: '−', 9: '+', 10: 'LS', 11: 'RS', 16: 'Home' },
  generic: { 4: 'LB', 5: 'RB', 6: 'LT', 7: 'RT', 8: 'Select', 9: 'Start', 10: 'L3', 11: 'R3', 16: 'Guide' },
};

export function gamepadButtonLabel(brand: GamepadBrand, index: number): string {
  if (DPAD_LABELS[index]) return DPAD_LABELS[index];
  if (brand === 'xbox' && XBOX_LABELS[index]) return XBOX_LABELS[index];
  if (brand === 'playstation' && PLAYSTATION_LABELS[index]) return PLAYSTATION_LABELS[index];
  if (brand === 'nintendo' && NINTENDO_LABELS[index]) return NINTENDO_LABELS[index];
  if (brand === 'generic' && XBOX_LABELS[index]) return XBOX_LABELS[index];
  return TEXT_LABELS[brand][index] ?? `Btn ${index}`;
}

// Real Kenney "Input Prompts" glyphs (CC0, kenney.nl), mirrored at
// github.com/Maaack/Kenney-Input-Prompts. `generic`/unrecognized pads use the
// Xbox set since that matches the W3C Standard Gamepad button layout browsers
// report for any controller.
const XBOX_ASSETS: Record<number, string> = {
  0: 'xbox/xbox_button_color_a.png',
  1: 'xbox/xbox_button_color_b.png',
  2: 'xbox/xbox_button_color_x.png',
  3: 'xbox/xbox_button_color_y.png',
  4: 'xbox/xbox_lb.png',
  5: 'xbox/xbox_rb.png',
  6: 'xbox/xbox_lt.png',
  7: 'xbox/xbox_rt.png',
  8: 'xbox/xbox_button_view.png',
  9: 'xbox/xbox_button_menu.png',
  10: 'xbox/xbox_ls.png',
  11: 'xbox/xbox_rs.png',
  12: 'xbox/xbox_dpad_up.png',
  13: 'xbox/xbox_dpad_down.png',
  14: 'xbox/xbox_dpad_left.png',
  15: 'xbox/xbox_dpad_right.png',
  16: 'xbox/xbox_guide.png',
};

const PLAYSTATION_ASSETS: Record<number, string> = {
  0: 'playstation/playstation_button_color_cross.png',
  1: 'playstation/playstation_button_color_circle.png',
  2: 'playstation/playstation_button_color_square.png',
  3: 'playstation/playstation_button_color_triangle.png',
  4: 'playstation/playstation_trigger_l1.png',
  5: 'playstation/playstation_trigger_r1.png',
  6: 'playstation/playstation_trigger_l2.png',
  7: 'playstation/playstation_trigger_r2.png',
  8: 'playstation/playstation4_button_share.png',
  9: 'playstation/playstation4_button_options.png',
  12: 'playstation/playstation_dpad_up.png',
  13: 'playstation/playstation_dpad_down.png',
  14: 'playstation/playstation_dpad_left.png',
  15: 'playstation/playstation_dpad_right.png',
};

const NINTENDO_ASSETS: Record<number, string> = {
  0: 'nintendo/switch_button_b.png',
  1: 'nintendo/switch_button_a.png',
  2: 'nintendo/switch_button_y.png',
  3: 'nintendo/switch_button_x.png',
  4: 'nintendo/switch_button_l.png',
  5: 'nintendo/switch_button_r.png',
  6: 'nintendo/switch_button_zl.png',
  7: 'nintendo/switch_button_zr.png',
  8: 'nintendo/switch_button_minus.png',
  9: 'nintendo/switch_button_plus.png',
  10: 'nintendo/switch_stick_l_press.png',
  11: 'nintendo/switch_stick_r_press.png',
  12: 'nintendo/switch_dpad_up.png',
  13: 'nintendo/switch_dpad_down.png',
  14: 'nintendo/switch_dpad_left.png',
  15: 'nintendo/switch_dpad_right.png',
  16: 'nintendo/switch_button_home.png',
};

const BRAND_ASSETS: Record<GamepadBrand, Record<number, string>> = {
  xbox: XBOX_ASSETS,
  generic: XBOX_ASSETS,
  playstation: PLAYSTATION_ASSETS,
  nintendo: NINTENDO_ASSETS,
};

function TextBadge({ size, label }: { size: number; label: string }) {
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        minWidth: size,
        height: size,
        padding: '0 0.375rem',
        borderRadius: size / 2,
        background: 'rgba(255,255,255,0.12)',
        border: '1.5px solid rgba(255,255,255,0.2)',
        color: '#fff',
        fontSize: size * 0.42,
        fontWeight: 700,
        lineHeight: 1,
        boxSizing: 'border-box',
      }}
    >
      {label}
    </span>
  );
}

export function GamepadButtonGlyph({ brand, index, size = 24 }: { brand: GamepadBrand; index: number; size?: number }) {
  const asset = BRAND_ASSETS[brand][index];
  if (asset) {
    return (
      <img
        src={assetUrl(`gamepad/${asset}`)}
        alt={gamepadButtonLabel(brand, index)}
        style={{ height: size * 1.35, width: 'auto', maxWidth: size * 2.2, objectFit: 'contain', flexShrink: 0 }}
      />
    );
  }
  return <TextBadge size={size} label={gamepadButtonLabel(brand, index)} />;
}
