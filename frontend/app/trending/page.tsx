import { Pagination } from "@/components/pagination";
import { TorrentCard } from "@/components/torrent-card";
import { getCategories, getFeatures, getTrending } from "@/lib/api";
import { getCopy, localizeCategoryLabel } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";

type TrendingPageProps = {
  searchParams: Promise<{
    category?: string;
    window?: string;
    page?: string;
    page_size?: string;
  }>;
};

const WINDOWS = ["24h", "72h", "7d"];

export default async function TrendingPage({ searchParams }: TrendingPageProps) {
  const { locale } = await getServerSitePreferences();
  const copy = getCopy(locale);
  const features = await getFeatures();
  if (!features.trending_enabled) {
    return (
      <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 text-center shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.trending.disabled}</h1>
      </section>
    );
  }

  const params = await searchParams;
  const category = params.category?.trim() || "all";
  const window = WINDOWS.includes(params.window || "") ? (params.window as string) : "24h";
  const page = Number(params.page || "1");
  const pageSize = Number(params.page_size || "20");

  const [categories, result] = await Promise.all([
    getCategories(),
    getTrending({
      window,
      category,
      page: Number.isFinite(page) && page > 0 ? page : 1,
      pageSize: Number.isFinite(pageSize) && pageSize > 0 ? pageSize : 20,
    }).catch(() => null),
  ]);

  if (!result) {
    return (
      <section className="mx-auto max-w-3xl border-4 border-ink bg-paper p-8 text-center shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.trending.unavailable}</h1>
      </section>
    );
  }

  return (
    <div className="w-full min-w-0 space-y-8">
      <section className="border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">{copy.trending.title}</h1>

        <form
          action="/trending"
          className="mt-4 grid gap-3 md:grid-cols-[auto_auto_auto] md:items-center"
        >
          <select
            name="window"
            defaultValue={window}
            className="border-2 border-ink bg-paper px-3 py-2 font-headline text-sm font-bold uppercase"
          >
            {WINDOWS.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>

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

          <button className="bauhaus-shadow-sm bauhaus-press border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase transition-all hover:bg-ink hover:text-paper">
            {copy.trending.apply}
          </button>
        </form>
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
            <p className="font-headline text-xl font-bold uppercase">{copy.trending.noData}</p>
          </article>
        )}
      </section>

      <Pagination
        pathname="/trending"
        page={result.page}
        pageSize={result.page_size}
        total={result.total}
        query={{ category, window }}
        labels={copy.pagination}
      />
    </div>
  );
}
