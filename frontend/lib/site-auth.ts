export const SITE_SESSION_COOKIE = "dht_site_session";
export const SITE_SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;

export function getPrivateModeEnabledFromEnv(): boolean {
  return parseEnvBool(process.env.PRIVATE_MODE_ENABLED);
}

export function getPrivateSitePasswordFromEnv(): string {
  return process.env.PRIVATE_SITE_PASSWORD?.trim() ?? "";
}

export function isPrivateModeActiveFromEnv(): boolean {
  return getPrivateModeEnabledFromEnv() && getPrivateSitePasswordFromEnv().length > 0;
}

function parseEnvBool(raw: string | undefined): boolean {
  if (!raw) return false;

  switch (raw.trim().toLowerCase()) {
    case "1":
    case "true":
    case "yes":
    case "on":
      return true;
    default:
      return false;
  }
}
