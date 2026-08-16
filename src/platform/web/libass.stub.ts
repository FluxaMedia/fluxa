export interface LibassRenderer {
  destroy(): Promise<void>;
}

export function createLibassRenderer(): LibassRenderer {
  throw new Error('libass renderer is only bundled for browser targets');
}
