import Link from "next/link";

import type { TorrentListItem } from "@/lib/api";
import { formatBytes, formatDate } from "@/lib/format";

export function TorrentCard({ item }: { item: TorrentListItem }) {
  return (
    <article className="border-4 border-ink bg-paper p-5 shadow-hard-sm">
      <header className="mb-3 flex items-start justify-between gap-4">
        <div>
          <Link
            href={`/torrent/${item.info_hash}`}
            className="font-headline text-xl font-black uppercase tracking-tight hover:underline"
          >
            {item.name || "Untitled"}
          </Link>
          <p className="mt-1 text-xs uppercase tracking-wide text-ink-muted">
            HASH {item.info_hash}
          </p>
        </div>
        <span className="border-2 border-ink bg-accent-yellow px-2 py-1 text-xs font-bold uppercase">
          {item.category}
        </span>
      </header>

      <div className="grid gap-2 text-sm text-ink md:grid-cols-2">
        <p>Size: {formatBytes(item.total_size)}</p>
        <p>Files: {item.file_count}</p>
        <p>Hot Score: {item.trend_score.toFixed(2)}</p>
        <p>Observations: {item.observations}</p>
        <p>First Seen: {formatDate(item.first_seen_at)}</p>
        <p>Last Seen: {formatDate(item.last_seen_at)}</p>
      </div>

      <footer className="mt-4 flex flex-wrap gap-3 text-sm">
        <a
          href={item.magnet_link}
          className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-accent-yellow px-3 py-1 font-bold uppercase tracking-wide transition-all hover:bg-ink hover:text-paper"
        >
          Magnet
        </a>
        <Link
          href={`/torrent/${item.info_hash}`}
          className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-paper px-3 py-1 font-bold uppercase tracking-wide transition-all hover:bg-accent-blue hover:text-paper"
        >
          Details
        </Link>
      </footer>
    </article>
  );
}
