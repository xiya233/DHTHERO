export function safeMagnetHref(raw: string): string | null {
  const value = raw.trim();
  if (!value) {
    return null;
  }

  if (!value.toLowerCase().startsWith("magnet:")) {
    return null;
  }

  return value;
}
