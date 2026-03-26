import { Pagination } from "@/components/pagination";
import { TorrentCard } from "@/components/torrent-card";
import { getCategories, searchTorrents } from "@/lib/api";
import { getCopy, localizeCategoryLabel } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

type SearchPageProps = {
  searchParams: Promise<{
    q?: string;
    category?: string;
    sort?: string;
    page?: string;
    page_size?: string;
  }>;
};

const SORT_KEYS = ["relevance", "latest", "hot", "size_desc", "size_asc"] as const;

export default async function SearchPage({ searchParams }: SearchPageProps) {
  const { locale } = await getServerSitePreferences();
  const copy = getCopy(locale);
  const params = await searchParams;
  const q = params.q?.trim() ?? "";
  const category = params.category?.trim() || "all";
  const sort = params.sort?.trim() || "relevance";
  const page = Number(params.page || "1");
  const pageSize = Number(params.page_size || "20");

  const categories = await getCategories();

  if (!q) {
    return (
      <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 text-center shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.search.emptyTitle}</h1>
        <p className="mt-3 text-ink-muted">{copy.search.emptyHint}</p>
      </section>
    );
  }

  const result = await searchTorrents({
    q,
    category,
    sort,
    page: Number.isFinite(page) && page > 0 ? page : 1,
    pageSize: Number.isFinite(pageSize) && pageSize > 0 ? pageSize : 20,
  }).catch(() => null);

  if (!result) {
    return (
      <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 text-center shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.search.failedTitle}</h1>
        <p className="mt-3 text-ink-muted">{copy.search.failedHint}</p>
      </section>
    );
  }

  return (
    <div className="w-full min-w-0 space-y-8">
      <section className="border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.search.resultTitle}</h1>
        <form action="/search" className="mt-5 grid gap-3 md:grid-cols-[1fr_auto_auto_auto]">
          <input
            name="q"
            defaultValue={q}
            required
            className="border-2 border-ink bg-paper px-3 py-2 font-headline text-lg font-bold uppercase outline-none"
          />

          <select
            name="category"
            defaultValue={category}
            className="border-2 border-ink bg-paper px-3 py-2 font-headline text-sm font-bold uppercase"
          >
            {categories.map((item) => (
              <option key={item.key} value={item.key}>
                {localizeCategoryLabel(locale, item.key, item.label)}
              </option>
            ))}
          </select>

          <select
            name="sort"
            defaultValue={sort}
            className="border-2 border-ink bg-paper px-3 py-2 font-headline text-sm font-bold uppercase"
          >
            {SORT_KEYS.map((sortKey) => (
              <option key={sortKey} value={sortKey}>
                {copy.search.sorts[sortKey]}
              </option>
            ))}
          </select>

          <button className="border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase hover:bg-ink hover:text-paper">
            {copy.search.apply}
          </button>
        </form>
        <p className="mt-4 text-sm uppercase tracking-wide text-ink-muted">
          {copy.search.summary(result.total, result.took_ms)}
        </p>
      </section>

      <section className="w-full min-w-0 space-y-4">
        {result.items.length > 0 ? (
          result.items.map((item) => (
            <TorrentCard
              key={item.info_hash}
              item={item}
              locale={locale}
              labels={copy.torrentCard}
            />
          ))
        ) : (
          <article className="border-4 border-ink bg-paper p-6 text-center shadow-hard-sm">
            <p className="font-headline text-xl font-bold uppercase">{copy.search.noResults}</p>
          </article>
        )}
      </section>

      <Pagination
        pathname="/search"
        page={result.page}
        pageSize={result.page_size}
        total={result.total}
        query={{ q, category, sort }}
        labels={copy.pagination}
      />
    </div>
  );
}
