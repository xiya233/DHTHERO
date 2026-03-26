import Link from "next/link";

import type { TorrentListItem } from "@/lib/api";
import { formatBytes, formatDate } from "@/lib/format";
import { localizeCategoryLabel, type SiteCopy } from "@/lib/i18n";
import type { SiteLocale } from "@/lib/site-preferences";
import { safeMagnetHref } from "@/lib/url-safety";

type TorrentCardProps = {
  item: TorrentListItem;
  locale?: SiteLocale;
  labels?: SiteCopy["torrentCard"];
};

export function TorrentCard({ item, locale = "en", labels }: TorrentCardProps) {
  const magnetHref = safeMagnetHref(item.magnet_link);
  const text = labels ?? {
    untitled: "Untitled",
    hash: "Hash",
    size: "Size",
    files: "Files",
    hotScore: "Hot Score",
    observations: "Observations",
    firstSeen: "First Seen",
    lastSeen: "Last Seen",
    magnet: "Magnet",
    details: "Details",
  };

  return (
    <article className="w-full min-w-0 border-4 border-ink bg-paper p-5 shadow-hard-sm">
      <header className="mb-3 flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <Link
            href={`/torrent/${item.info_hash}`}
            className="clamp-2 break-all font-headline text-xl font-black uppercase tracking-tight hover:underline"
          >
            {item.name || text.untitled}
          </Link>
          <p className="mt-1 break-all text-xs uppercase tracking-wide text-ink-muted">
            {text.hash} {item.info_hash}
          </p>
        </div>
        <span className="shrink-0 border-2 border-ink bg-accent-yellow px-2 py-1 text-xs font-bold uppercase">
          {localizeCategoryLabel(locale, item.category, item.category)}
        </span>
      </header>

      <div className="grid gap-2 text-sm text-ink md:grid-cols-2">
        <p>
          {text.size}: {formatBytes(item.total_size)}
        </p>
        <p>
          {text.files}: {item.file_count}
        </p>
        <p>
          {text.hotScore}: {item.trend_score.toFixed(2)}
        </p>
        <p>
          {text.observations}: {item.observations}
        </p>
        <p>
          {text.firstSeen}: {formatDate(item.first_seen_at)}
        </p>
        <p>
          {text.lastSeen}: {formatDate(item.last_seen_at)}
        </p>
      </div>

      <footer className="mt-4 flex flex-wrap gap-3 text-sm">
        {magnetHref ? (
          <a
            href={magnetHref}
            className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-accent-yellow px-3 py-1 font-bold uppercase tracking-wide transition-all hover:bg-ink hover:text-paper"
          >
            {text.magnet}
          </a>
        ) : (
          <span
            aria-disabled="true"
            className="inline-flex cursor-not-allowed items-center justify-center border-2 border-ink bg-accent-yellow px-3 py-1 font-bold uppercase tracking-wide opacity-60"
          >
            {text.magnet}
          </span>
        )}
        <Link
          href={`/torrent/${item.info_hash}`}
          className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-paper px-3 py-1 font-bold uppercase tracking-wide transition-all hover:bg-accent-blue hover:text-paper"
        >
          {text.details}
        </Link>
      </footer>
    </article>
  );
}
