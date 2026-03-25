import { cookies, headers } from "next/headers";

import {
  SITE_LOCALE_KEY,
  SITE_THEME_KEY,
  detectLocaleFromAcceptLanguage,
  normalizeSiteLocale,
  normalizeSiteTheme,
  type SiteLocale,
  type SiteTheme,
} from "@/lib/site-preferences";

export async function getServerSitePreferences(): Promise<{
  locale: SiteLocale;
  theme: SiteTheme;
}> {
  const cookieStore = await cookies();
  const headerStore = await headers();

  const cookieLocale = normalizeSiteLocale(cookieStore.get(SITE_LOCALE_KEY)?.value);
  const locale = cookieLocale ?? detectLocaleFromAcceptLanguage(headerStore.get("accept-language"));

  const cookieTheme = normalizeSiteTheme(cookieStore.get(SITE_THEME_KEY)?.value);
  const theme = cookieTheme ?? "light";

  return { locale, theme };
}
