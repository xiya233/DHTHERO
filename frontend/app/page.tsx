import Link from "next/link";
import ReactMarkdown from "react-markdown";

import { BauhausIcon } from "@/components/bauhaus-icon";
import { getCategories, getFeatures, getSiteContent, getSiteStats } from "@/lib/api";
import { formatCompactNumber, formatNumber } from "@/lib/format";
import { getCopy } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

const FIXED_CATEGORY_ORDER = [
  { key: "all", icon: "all" },
  { key: "video", icon: "video" },
  { key: "audio", icon: "audio" },
  { key: "doc", icon: "doc" },
  { key: "app", icon: "app" },
  { key: "other", icon: "other" },
] as const;

function heroMarkdownForRender(markdown: string): string {
  const normalized = markdown.trim().replace(/\r\n?/g, "\n");
  if (!normalized) {
    return "SEARCH  \nTHE_NET";
  }
  return normalized.replace(/\n/g, "  \n");
}

export default async function HomePage() {
  const [{ locale }, features, stats, categories, siteContent] = await Promise.all([
    getServerSitePreferences(),
    getFeatures(),
    getSiteStats(),
    getCategories(),
    getSiteContent(),
  ]);
  const copy = getCopy(locale);

  const categoryCountMap = new Map(categories.map((item) => [item.key, item.count]));
  const categoryItems = FIXED_CATEGORY_ORDER.map((item) => ({
    ...item,
    label: copy.home.categories[item.key],
    count: categoryCountMap.get(item.key) ?? 0,
  }));

  return (
    <>
      <section className="mb-16 w-full max-w-4xl text-center">
        <div className="mb-4 font-headline text-7xl font-black uppercase leading-none tracking-tighter md:text-9xl">
          <ReactMarkdown
            components={{
              p: ({ children }) => <p className="m-0">{children}</p>,
            }}
          >
            {heroMarkdownForRender(siteContent.home_hero_markdown)}
          </ReactMarkdown>
        </div>
        <div className="flex justify-center">
          <div className="inline-block border-2 border-ink bg-ink px-4 py-1 font-headline text-sm font-bold uppercase tracking-widest text-paper">
            {copy.home.totalIndexed}: {formatNumber(stats?.total_torrents_indexed ?? 0)}
          </div>
        </div>
      </section>

      <section className="mb-12 w-full max-w-3xl">
        {features.search_enabled ? (
          <form
            action="/search"
            method="get"
            className="bauhaus-shadow bauhaus-press flex cursor-pointer flex-col gap-0 border-4 border-ink bg-paper transition-all md:flex-row"
          >
            <div className="flex flex-grow items-center bg-paper-soft px-6 py-4">
              <BauhausIcon name="search" className="mr-4 size-8" />
              <input
                name="q"
                type="text"
                required
                placeholder={copy.home.searchPlaceholder}
                className="w-full border-none bg-transparent font-headline text-2xl font-bold uppercase outline-none placeholder:text-ink-muted"
              />
            </div>

            <button
              type="submit"
              className="border-t-4 border-ink bg-accent-yellow px-12 py-6 font-headline text-3xl font-black uppercase tracking-tighter transition-colors hover:bg-ink hover:text-paper active:translate-x-0 active:translate-y-0 md:border-t-0 md:border-l-4"
            >
              {copy.home.searchButton}
            </button>
          </form>
        ) : (
          <div className="bauhaus-shadow-sm border-4 border-ink bg-paper p-6 text-center">
            <p className="font-headline text-lg font-bold uppercase">
              {copy.home.searchDisabled}
            </p>
          </div>
        )}
      </section>

      <section className="mb-20 flex max-w-2xl flex-wrap justify-center gap-4">
        {categoryItems.map((item) => {
          const href =
            item.key === "all"
              ? "/latest"
              : `/latest?category=${encodeURIComponent(item.key)}`;
          const isAll = item.key === "all";
          const compactCount = item.count > 0 ? formatCompactNumber(item.count) : null;
          const fullCountLabel = item.count > 0 ? `${item.label} (${formatNumber(item.count)})` : item.label;
          return (
            <Link
              key={item.key}
              href={href}
              title={fullCountLabel}
              className={`bauhaus-shadow-sm bauhaus-press inline-flex h-12 w-[168px] items-center justify-center gap-2 overflow-hidden whitespace-nowrap border-2 border-ink px-3 py-2 font-headline text-sm font-bold uppercase tracking-wider transition-all ${
                isAll
                  ? "bg-ink text-paper"
                  : "bg-paper-soft hover:bg-accent-blue hover:text-paper"
              }`}
            >
              <BauhausIcon name={item.icon} className="size-4 shrink-0" />
              <span className="min-w-0 truncate">
                {item.label}
                {compactCount ? <span className="tabular-nums"> ({compactCount})</span> : null}
              </span>
            </Link>
          );
        })}
      </section>

      <section className="mb-12 grid w-full max-w-6xl grid-cols-1 gap-8 md:grid-cols-3">
        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-paper-soft p-6">
          <div className="absolute top-0 right-0 h-16 w-16 border-b-4 border-l-4 border-ink bg-accent-red" />
          <h3 className="relative z-10 mb-4 font-headline text-3xl font-black uppercase">
            {copy.home.cardFastTitle}
          </h3>
          <p className="relative z-10 text-sm leading-relaxed">
            {copy.home.cardFastBody}
          </p>
          <div className="mt-8 h-1 w-full bg-ink" />
        </article>

        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-accent-yellow p-6">
          <h3 className="mb-4 font-headline text-3xl font-black uppercase">{copy.home.cardNoLogsTitle}</h3>
          <p className="text-sm leading-relaxed">{copy.home.cardNoLogsBody}</p>
          <div className="mt-8 flex gap-2">
            <div className="h-4 w-4 bg-accent-red" />
            <div className="h-4 w-4 bg-accent-blue" />
            <div className="h-4 w-4 bg-ink" />
          </div>
        </article>

        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-paper-soft p-6">
          <div className="absolute bottom-0 left-0 h-4 w-24 bg-accent-blue" />
          <h3 className="mb-4 font-headline text-3xl font-black uppercase">{copy.home.cardP2PTitle}</h3>
          <p className="text-sm leading-relaxed">{copy.home.cardP2PBody}</p>
          <div className="mt-8 text-right font-headline text-xs font-bold uppercase tracking-tighter">
            V. 2.4.0-BAUHAUS
          </div>
        </article>
      </section>
    </>
  );
}
