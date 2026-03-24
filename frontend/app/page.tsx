import Link from "next/link";

import { getCategories, getFeatures, getSiteStats } from "@/lib/api";
import { formatNumber } from "@/lib/format";

export default async function HomePage() {
  const [features, stats, categories] = await Promise.all([
    getFeatures(),
    getSiteStats(),
    getCategories(),
  ]);

  return (
    <div className="space-y-14">
      <section className="space-y-5 text-center">
        <h1 className="font-headline text-6xl font-black uppercase leading-none tracking-tight md:text-8xl">
          Search
          <br />
          The_Net
        </h1>

        <p className="inline-block border-2 border-ink bg-ink px-4 py-1 text-sm font-bold uppercase tracking-widest text-paper">
          Total Torrents Indexed: {formatNumber(stats?.total_torrents_indexed ?? 0)}
        </p>
      </section>

      <section className="mx-auto max-w-4xl">
        {features.search_enabled ? (
          <form
            action="/search"
            method="get"
            className="shadow-hard flex flex-col overflow-hidden border-4 border-ink bg-paper md:flex-row"
          >
            <input
              name="q"
              required
              placeholder="MAGNET HASH OR FILENAME..."
              className="w-full bg-paper px-5 py-4 font-headline text-xl font-bold uppercase outline-none placeholder:text-ink-muted"
            />
            <button
              type="submit"
              className="border-t-4 border-ink bg-accent-yellow px-10 py-4 font-headline text-2xl font-black uppercase tracking-tight transition hover:bg-ink hover:text-paper md:border-t-0 md:border-l-4"
            >
              Search
            </button>
          </form>
        ) : (
          <div className="border-4 border-ink bg-paper p-6 text-center shadow-hard-sm">
            <p className="font-headline text-lg font-bold uppercase">
              Search feature is disabled by environment flag.
            </p>
          </div>
        )}
      </section>

      <section className="flex flex-wrap justify-center gap-3">
        {categories
          .filter((item) => item.key !== "all")
          .map((item) => (
            <Link
              key={item.key}
              href={`/latest?category=${item.key}`}
              className="shadow-hard-sm border-2 border-ink bg-paper px-4 py-2 font-headline text-sm font-bold uppercase tracking-wide hover:bg-accent-blue hover:text-paper"
            >
              {item.label} ({formatNumber(item.count)})
            </Link>
          ))}
      </section>

      <section className="grid gap-6 md:grid-cols-3">
        <article className="shadow-hard-sm border-4 border-ink bg-paper p-6">
          <h2 className="font-headline text-3xl font-black uppercase">Fast_Indexing</h2>
          <p className="mt-3 text-sm leading-relaxed text-ink-muted">
            Real-time DHT crawling with modular Rust workers and PostgreSQL indexing pipeline.
          </p>
        </article>
        <article className="shadow-hard-sm border-4 border-ink bg-accent-yellow p-6">
          <h2 className="font-headline text-3xl font-black uppercase">Audit_Ready</h2>
          <p className="mt-3 text-sm leading-relaxed">
            Full search audit logging is enabled via environment switches and partitioned storage.
          </p>
        </article>
        <article className="shadow-hard-sm border-4 border-ink bg-paper p-6">
          <h2 className="font-headline text-3xl font-black uppercase">P2P_Power</h2>
          <p className="mt-3 text-sm leading-relaxed text-ink-muted">
            Trending scores derive from DHT observations, not centralized query analytics.
          </p>
        </article>
      </section>
    </div>
  );
}
