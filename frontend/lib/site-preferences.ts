export type SiteLocale = "en" | "zh";
export type SiteTheme = "light" | "dark";

export const SITE_LOCALE_KEY = "site_locale";
export const SITE_THEME_KEY = "site_theme";

const LOCALE_VALUES: SiteLocale[] = ["en", "zh"];
const THEME_VALUES: SiteTheme[] = ["light", "dark"];

export function normalizeSiteLocale(raw: string | null | undefined): SiteLocale | null {
  const value = raw?.trim().toLowerCase();
  if (!value) {
    return null;
  }

  if (LOCALE_VALUES.includes(value as SiteLocale)) {
    return value as SiteLocale;
  }

  if (value.startsWith("zh")) {
    return "zh";
  }

  if (value.startsWith("en")) {
    return "en";
  }

  return null;
}

export function normalizeSiteTheme(raw: string | null | undefined): SiteTheme | null {
  const value = raw?.trim().toLowerCase();
  if (!value) {
    return null;
  }

  if (THEME_VALUES.includes(value as SiteTheme)) {
    return value as SiteTheme;
  }

  return null;
}

export function detectLocaleFromAcceptLanguage(raw: string | null | undefined): SiteLocale {
  const normalized = raw?.toLowerCase() ?? "";
  return normalized.includes("zh") ? "zh" : "en";
}
