import Link from "next/link";

import { BauhausIcon } from "@/components/bauhaus-icon";
import { getCategories, getFeatures, getSiteStats } from "@/lib/api";
import { formatNumber } from "@/lib/format";

const FIXED_CATEGORY_ORDER = [
  { key: "all", label: "All" },
  { key: "video", label: "Video" },
  { key: "audio", label: "Audio" },
  { key: "doc", label: "Doc" },
  { key: "app", label: "App" },
  { key: "other", label: "Other" },
];

export default async function HomePage() {
  const [features, stats, categories] = await Promise.all([
    getFeatures(),
    getSiteStats(),
    getCategories(),
  ]);

  const categoryCountMap = new Map(categories.map((item) => [item.key, item.count]));
  const categoryItems = FIXED_CATEGORY_ORDER.map((item) => ({
    ...item,
    count: categoryCountMap.get(item.key) ?? 0,
  }));

  return (
    <>
      <section className="mb-16 w-full max-w-4xl text-center">
        <h1 className="mb-4 font-headline text-7xl font-black uppercase leading-none tracking-tighter md:text-9xl">
          SEARCH
          <br />
          THE_NET
        </h1>
        <div className="flex justify-center">
          <div className="inline-block border-2 border-ink bg-ink px-4 py-1 font-headline text-sm font-bold uppercase tracking-widest text-paper">
            Total Torrents Indexed: {formatNumber(stats?.total_torrents_indexed ?? 0)}
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
            <div className="flex flex-grow items-center bg-white px-6 py-4">
              <BauhausIcon name="search" className="mr-4 size-8" />
              <input
                name="q"
                type="text"
                required
                placeholder="MAGNET HASH OR FILENAME..."
                className="w-full border-none bg-transparent font-headline text-2xl font-bold uppercase outline-none placeholder:text-ink-muted"
              />
            </div>

            <button
              type="submit"
              className="border-t-4 border-ink bg-accent-yellow px-12 py-6 font-headline text-3xl font-black uppercase tracking-tighter transition-colors hover:bg-ink hover:text-paper active:translate-x-0 active:translate-y-0 md:border-t-0 md:border-l-4"
            >
              SEARCH
            </button>
          </form>
        ) : (
          <div className="bauhaus-shadow-sm border-4 border-ink bg-paper p-6 text-center">
            <p className="font-headline text-lg font-bold uppercase">
              Search feature is disabled by environment flag.
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
          return (
            <Link
              key={item.key}
              href={href}
              className={`bauhaus-shadow-sm border-2 border-ink px-6 py-2 font-headline text-sm font-bold uppercase tracking-wider transition-all active:translate-y-0.5 active:shadow-none ${
                isAll
                  ? "bg-ink text-paper"
                  : "bg-white hover:bg-accent-blue hover:text-paper"
              }`}
            >
              {item.label}
              {item.count > 0 ? ` (${formatNumber(item.count)})` : ""}
            </Link>
          );
        })}
      </section>

      <section className="mb-12 grid w-full max-w-6xl grid-cols-1 gap-8 md:grid-cols-3">
        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-white p-6">
          <div className="absolute top-0 right-0 h-16 w-16 border-b-4 border-l-4 border-ink bg-accent-red" />
          <h3 className="relative z-10 mb-4 font-headline text-3xl font-black uppercase">
            Fast_Indexing
          </h3>
          <p className="relative z-10 text-sm leading-relaxed">
            Real-time DHT crawler mapping the global decentralized network. Zero-latency search for
            the most elusive magnet links.
          </p>
          <div className="mt-8 h-1 w-full bg-ink" />
        </article>

        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-accent-yellow p-6">
          <h3 className="mb-4 font-headline text-3xl font-black uppercase">No_Logs</h3>
          <p className="text-sm leading-relaxed">
            Privacy by design. We don&apos;t track your queries. The network is yours to explore
            without footprints or centralized interference.
          </p>
          <div className="mt-8 flex gap-2">
            <div className="h-4 w-4 bg-accent-red" />
            <div className="h-4 w-4 bg-accent-blue" />
            <div className="h-4 w-4 bg-ink" />
          </div>
        </article>

        <article className="bauhaus-shadow relative overflow-hidden border-4 border-ink bg-white p-6">
          <div className="absolute bottom-0 left-0 h-4 w-24 bg-accent-blue" />
          <h3 className="mb-4 font-headline text-3xl font-black uppercase">P2P_Power</h3>
          <p className="text-sm leading-relaxed">
            Harnessing the strength of distributed systems. No central server dependency for data
            integrity or availability.
          </p>
          <div className="mt-8 text-right font-headline text-xs font-bold uppercase tracking-tighter">
            V. 2.4.0-BAUHAUS
          </div>
        </article>
      </section>
    </>
  );
}
