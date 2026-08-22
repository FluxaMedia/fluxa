import { platformInvoke as invoke } from '../platform/invoke';
import { platformOpenDialog } from '../platform/browser';

export interface CustomFont {
  fileName: string;
  family: string;
}

export async function listCustomFonts(): Promise<CustomFont[]> {
  return invoke<CustomFont[]>('custom_fonts_list').catch(() => []);
}

export async function removeCustomFont(fileName: string): Promise<void> {
  return invoke('custom_fonts_remove', { fileName });
}

export async function pickAndAddCustomFont(): Promise<CustomFont | null> {
  const selected = await platformOpenDialog({
    multiple: false,
    filters: [{ name: 'Fonts', extensions: ['ttf', 'otf', 'ttc'] }],
  });
  if (!selected || Array.isArray(selected)) return null;
  return invoke<CustomFont>('custom_fonts_add', { sourcePath: selected });
}
