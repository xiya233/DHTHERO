export const ADMIN_SESSION_COOKIE = "dht_admin_session";
export const ADMIN_SESSION_TTL_SECONDS = 60 * 60 * 24;

export function getAdminPasswordFromEnv(): string {
  return process.env.ADMIN_DASHBOARD_PASSWORD?.trim() ?? "";
}
