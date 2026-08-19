import type { ReactNode } from 'react';

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

const XBOX_FACE: Record<number, { label: string; color: string }> = {
  0: { label: 'A', color: '#5FA341' },
  1: { label: 'B', color: '#C74440' },
  2: { label: 'X', color: '#3A7DC9' },
  3: { label: 'Y', color: '#D8A928' },
};

const NINTENDO_FACE: Record<number, string> = {
  0: 'B',
  1: 'A',
  2: 'Y',
  3: 'X',
};

const TEXT_LABELS: Record<GamepadBrand, Record<number, string>> = {
  xbox: { 4: 'LB', 5: 'RB', 6: 'LT', 7: 'RT', 8: 'View', 9: 'Menu', 10: 'LS', 11: 'RS', 16: 'Guide' },
  playstation: { 4: 'L1', 5: 'R1', 6: 'L2', 7: 'R2', 8: 'Share', 9: 'Options', 10: 'L3', 11: 'R3', 16: 'PS' },
  nintendo: { 4: 'L', 5: 'R', 6: 'ZL', 7: 'ZR', 8: '−', 9: '+', 10: 'LS', 11: 'RS', 16: 'Home' },
  generic: {},
};

const PLAYSTATION_FACE_LABELS: Record<number, string> = { 0: 'Cross', 1: 'Circle', 2: 'Square', 3: 'Triangle' };
const DPAD_LABELS: Record<number, string> = { 12: '↑', 13: '↓', 14: '←', 15: '→' };

export function gamepadButtonLabel(brand: GamepadBrand, index: number): string {
  if (DPAD_LABELS[index]) return DPAD_LABELS[index];
  if (brand === 'xbox' && XBOX_FACE[index]) return XBOX_FACE[index].label;
  if (brand === 'playstation' && PLAYSTATION_FACE_LABELS[index]) return PLAYSTATION_FACE_LABELS[index];
  if (brand === 'nintendo' && NINTENDO_FACE[index]) return NINTENDO_FACE[index];
  return TEXT_LABELS[brand][index] ?? `Btn ${index}`;
}

function Pill({
  size,
  children,
  background = 'rgba(255,255,255,0.12)',
  color = '#fff',
  border = 'rgba(255,255,255,0.2)',
}: {
  size: number;
  children: ReactNode;
  background?: string;
  color?: string;
  border?: string;
}) {
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
        background,
        border: `1px solid ${border}`,
        color,
        fontSize: size * 0.42,
        fontWeight: 700,
        lineHeight: 1,
        boxSizing: 'border-box',
      }}
    >
      {children}
    </span>
  );
}

function DpadArrow({ size, direction }: { size: number; direction: 'up' | 'down' | 'left' | 'right' }) {
  const rotation = { up: 0, right: 90, down: 180, left: 270 }[direction];
  return (
    <Pill size={size}>
      <svg width={size * 0.4} height={size * 0.4} viewBox="0 0 24 24" style={{ transform: `rotate(${rotation}deg)` }}>
        <path d="M12 3 L21 18 L3 18 Z" fill="currentColor" />
      </svg>
    </Pill>
  );
}

function PlayStationShape({ size, index }: { size: number; index: number }) {
  const strokeWidth = size * 0.09;
  const inner = size * 0.42;
  const common = { width: inner, height: inner, viewBox: '0 0 24 24' };
  if (index === 0) {
    return (
      <Pill size={size} color="#F16FA8">
        <svg {...common}>
          <path d="M5 5 L19 19 M19 5 L5 19" stroke="currentColor" strokeWidth={strokeWidth * 2.4} fill="none" strokeLinecap="round" />
        </svg>
      </Pill>
    );
  }
  if (index === 1) {
    return (
      <Pill size={size} color="#EF4A4A">
        <svg {...common}>
          <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth={strokeWidth * 2.4} fill="none" />
        </svg>
      </Pill>
    );
  }
  if (index === 2) {
    return (
      <Pill size={size} color="#F0A6D0">
        <svg {...common}>
          <rect x="5" y="5" width="14" height="14" stroke="currentColor" strokeWidth={strokeWidth * 2.4} fill="none" />
        </svg>
      </Pill>
    );
  }
  return (
    <Pill size={size} color="#4FBF8B">
      <svg {...common}>
        <path d="M12 4 L20 19 L4 19 Z" stroke="currentColor" strokeWidth={strokeWidth * 2.4} fill="none" strokeLinejoin="round" />
      </svg>
    </Pill>
  );
}

export function GamepadButtonGlyph({ brand, index, size = 24 }: { brand: GamepadBrand; index: number; size?: number }) {
  if (index >= 12 && index <= 15) {
    const direction = ({ 12: 'up', 13: 'down', 14: 'left', 15: 'right' } as const)[index as 12 | 13 | 14 | 15];
    return <DpadArrow size={size} direction={direction} />;
  }

  if (brand === 'xbox' && XBOX_FACE[index]) {
    const face = XBOX_FACE[index];
    return (
      <Pill size={size} background={face.color} color="#fff" border={face.color}>
        {face.label}
      </Pill>
    );
  }

  if (brand === 'playstation' && index >= 0 && index <= 3) {
    return <PlayStationShape size={size} index={index} />;
  }

  if (brand === 'nintendo' && NINTENDO_FACE[index]) {
    return <Pill size={size}>{NINTENDO_FACE[index]}</Pill>;
  }

  const textLabel = TEXT_LABELS[brand][index];
  if (textLabel) return <Pill size={size}>{textLabel}</Pill>;

  return <Pill size={size}>{index}</Pill>;
}
