"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { BauhausIcon } from "@/components/bauhaus-icon";
import { getCopy } from "@/lib/i18n";
import {
  SITE_LOCALE_KEY,
  SITE_THEME_KEY,
  normalizeSiteLocale,
  normalizeSiteTheme,
  type SiteLocale,
  type SiteTheme,
} from "@/lib/site-preferences";

const PREFERENCE_MAX_AGE_SECONDS = 60 * 60 * 24 * 365;

function persistPreference(key: string, value: string) {
  document.cookie = `${key}=${encodeURIComponent(value)}; path=/; max-age=${PREFERENCE_MAX_AGE_SECONDS}; samesite=lax`;
  window.localStorage.setItem(key, value);
}

type SitePreferenceControlsProps = {
  initialLocale: SiteLocale;
  initialTheme: SiteTheme;
};

export function SitePreferenceControls({
  initialLocale,
  initialTheme,
}: SitePreferenceControlsProps) {
  const router = useRouter();
  const [locale, setLocale] = useState<SiteLocale>(() => {
    if (typeof document === "undefined") {
      return initialLocale;
    }

    const domLocale = normalizeSiteLocale(document.documentElement.lang);
    const storedLocale = normalizeSiteLocale(window.localStorage.getItem(SITE_LOCALE_KEY));
    return domLocale ?? storedLocale ?? initialLocale;
  });
  const [theme, setTheme] = useState<SiteTheme>(() => {
    if (typeof document === "undefined") {
      return initialTheme;
    }

    const domTheme = normalizeSiteTheme(document.documentElement.getAttribute("data-theme"));
    const storedTheme = normalizeSiteTheme(window.localStorage.getItem(SITE_THEME_KEY));
    return domTheme ?? storedTheme ?? initialTheme;
  });
  const copy = useMemo(() => getCopy(locale), [locale]);

  useEffect(() => {
    document.documentElement.lang = locale;
    document.documentElement.setAttribute("data-theme", theme);
  }, [locale, theme]);

  const onToggleLocale = () => {
    const nextLocale: SiteLocale = locale === "en" ? "zh" : "en";
    setLocale(nextLocale);
    persistPreference(SITE_LOCALE_KEY, nextLocale);
    document.documentElement.lang = nextLocale;
    window.dispatchEvent(new CustomEvent<SiteLocale>("site-locale-change", { detail: nextLocale }));
    router.refresh();
  };

  const onToggleTheme = () => {
    const nextTheme: SiteTheme = theme === "light" ? "dark" : "light";
    setTheme(nextTheme);
    persistPreference(SITE_THEME_KEY, nextTheme);
    document.documentElement.setAttribute("data-theme", nextTheme);
    window.dispatchEvent(new CustomEvent<SiteTheme>("site-theme-change", { detail: nextTheme }));
  };

  return (
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onToggleLocale}
        aria-label={copy.layout.langSwitchAria}
        className="bauhaus-shadow-sm bauhaus-press inline-flex items-center gap-1 border-2 border-ink bg-paper px-2 py-1 font-headline text-xs font-bold uppercase transition-all hover:bg-accent-yellow"
      >
        <BauhausIcon name="language" className="size-4" />
        <span>文/A</span>
        <span className="text-[10px]">{copy.layout.langButtonLabel}</span>
      </button>

      <button
        type="button"
        onClick={onToggleTheme}
        aria-label={copy.layout.themeSwitchAria}
        className="bauhaus-shadow-sm bauhaus-press inline-flex border-2 border-ink bg-paper p-2 transition-all hover:bg-accent-yellow"
      >
        <BauhausIcon
          name={theme === "dark" ? "light_mode" : "dark_mode"}
          className="size-5"
        />
      </button>
    </div>
  );
}
