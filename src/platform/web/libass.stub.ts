export interface LibassRenderer {
  destroy(): Promise<void>;
}

export function createLibassRenderer(..._args: unknown[]): LibassRenderer {
  throw new Error('libass renderer is only bundled for browser targets');
}
