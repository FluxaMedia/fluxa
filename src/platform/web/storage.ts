export function storageRead(key: string): string | null {
  return localStorage.getItem(key);
}

export function storageWrite(key: string, value: string): boolean {
  localStorage.setItem(key, value);
  return true;
}

export function storageDelete(key: string): boolean {
  localStorage.removeItem(key);
  return true;
}
