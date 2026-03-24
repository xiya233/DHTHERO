export const ADMIN_SESSION_COOKIE = "dht_admin_session";

export function getAdminPasswordFromEnv(): string {
  return process.env.ADMIN_DASHBOARD_PASSWORD?.trim() ?? "";
}
