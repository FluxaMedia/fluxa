export class AsyncScope {
  private revision = 0;

  capture(): number {
    return this.revision;
  }

  invalidate(): void {
    this.revision += 1;
  }

  advance(): number {
    this.invalidate();
    return this.revision;
  }

  isCurrent(revision: number): boolean {
    return revision === this.revision;
  }
}
