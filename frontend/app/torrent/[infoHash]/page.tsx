import { notFound } from "next/navigation";

import { getFeatures, getTorrentDetail, getTorrentFiles } from "@/lib/api";
import { formatBytes, formatDate } from "@/lib/format";
import { getCopy, localizeCategoryLabel } from "@/lib/i18n";
import { getServerSitePreferences } from "@/lib/site-preferences-server";
import { safeMagnetHref } from "@/lib/url-safety";

type TorrentDetailPageProps = {
  params: Promise<{
    infoHash: string;
  }>;
};

export default async function TorrentDetailPage({ params }: TorrentDetailPageProps) {
  const [{ locale }, { infoHash }] = await Promise.all([
    getServerSitePreferences(),
    params,
  ]);
  const copy = getCopy(locale);
  const [detail, features] = await Promise.all([getTorrentDetail(infoHash), getFeatures()]);
  if (!detail) {
    notFound();
  }

  const files = features.file_tree_enabled
    ? await getTorrentFiles(detail.info_hash, false)
    : detail.files_preview;
  const magnetHref = safeMagnetHref(detail.magnet_link);

  return (
    <div className="w-full min-w-0 space-y-8">
      <section className="w-full min-w-0 border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h1 className="min-w-0 break-all font-headline text-4xl font-black uppercase tracking-tight">
          {detail.name}
        </h1>
        <p className="mt-2 break-all text-xs uppercase tracking-widest text-ink-muted">
          {detail.info_hash}
        </p>

        <div className="mt-5 grid gap-3 text-sm md:grid-cols-2">
          <p>
            {copy.torrent.category}: {localizeCategoryLabel(locale, detail.category, detail.category)}
          </p>
          <p>
            {copy.torrent.totalSize}: {formatBytes(detail.total_size)}
          </p>
          <p>
            {copy.torrent.pieceLength}: {formatBytes(detail.piece_length)}
          </p>
          <p>
            {copy.torrent.fileCount}: {detail.file_count}
          </p>
          <p>
            {copy.torrent.firstSeen}: {formatDate(detail.first_seen_at)}
          </p>
          <p>
            {copy.torrent.lastSeen}: {formatDate(detail.last_seen_at)}
          </p>
          <p>
            {copy.torrent.hotScore}: {detail.hot_score.toFixed(2)}
          </p>
        </div>

        <div className="mt-6">
          {magnetHref ? (
            <a
              href={magnetHref}
              className="bauhaus-shadow-sm bauhaus-press inline-flex border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase tracking-wider transition-all hover:bg-ink hover:text-paper"
            >
              {copy.torrent.openMagnet}
            </a>
          ) : (
            <span
              aria-disabled="true"
              className="inline-flex cursor-not-allowed border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase tracking-wider opacity-60"
            >
              {copy.torrent.openMagnet}
            </span>
          )}
        </div>
      </section>

      <section className="w-full min-w-0 border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h2 className="font-headline text-3xl font-black uppercase">{copy.torrent.files}</h2>
        {files.length > 0 ? (
          <ul className="mt-4 w-full min-w-0 space-y-2 text-sm">
            {files.slice(0, 300).map((file) => (
              <li key={`${file.path}-${file.is_dir ? "dir" : "file"}`}>
                <div
                  className="flex w-full min-w-0 items-center justify-between border-2 border-ink bg-paper-soft px-3 py-2"
                  style={{ paddingLeft: `${0.75 + file.depth * 0.8}rem` }}
                >
                  <span className="min-w-0 flex-1 truncate uppercase tracking-wide">
                    {file.is_dir ? `${copy.torrent.dirPrefix} ${file.path}` : file.path}
                  </span>
                  <span className="ml-3 shrink-0 text-xs text-ink-muted">
                    {file.is_dir ? "-" : formatBytes(file.size)}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p className="mt-4 text-sm text-ink-muted">{copy.torrent.noFiles}</p>
        )}
      </section>
    </div>
  );
}
