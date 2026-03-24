import { notFound } from "next/navigation";

import { getFeatures, getTorrentDetail, getTorrentFiles } from "@/lib/api";
import { formatBytes, formatDate } from "@/lib/format";

type TorrentDetailPageProps = {
  params: Promise<{
    infoHash: string;
  }>;
};

export default async function TorrentDetailPage({ params }: TorrentDetailPageProps) {
  const { infoHash } = await params;
  const detail = await getTorrentDetail(infoHash);
  if (!detail) {
    notFound();
  }

  const features = await getFeatures();
  const files = features.file_tree_enabled
    ? await getTorrentFiles(detail.info_hash, false)
    : detail.files_preview;

  return (
    <div className="space-y-8">
      <section className="border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase tracking-tight">
          {detail.name}
        </h1>
        <p className="mt-2 text-xs uppercase tracking-widest text-ink-muted">{detail.info_hash}</p>

        <div className="mt-5 grid gap-3 text-sm md:grid-cols-2">
          <p>Category: {detail.category}</p>
          <p>Total Size: {formatBytes(detail.total_size)}</p>
          <p>Piece Length: {formatBytes(detail.piece_length)}</p>
          <p>File Count: {detail.file_count}</p>
          <p>First Seen: {formatDate(detail.first_seen_at)}</p>
          <p>Last Seen: {formatDate(detail.last_seen_at)}</p>
          <p>Hot Score: {detail.hot_score.toFixed(2)}</p>
        </div>

        <div className="mt-6">
          <a
            href={detail.magnet_link}
            className="inline-flex border-2 border-ink bg-accent-yellow px-4 py-2 font-headline text-sm font-black uppercase tracking-wider hover:bg-ink hover:text-paper"
          >
            Open Magnet Link
          </a>
        </div>
      </section>

      <section className="border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <h2 className="font-headline text-3xl font-black uppercase">Files</h2>
        {files.length > 0 ? (
          <ul className="mt-4 space-y-2 text-sm">
            {files.slice(0, 300).map((file) => (
              <li key={`${file.path}-${file.is_dir ? "dir" : "file"}`}>
                <div
                  className="flex items-center justify-between border-2 border-ink bg-paper-soft px-3 py-2"
                  style={{ paddingLeft: `${0.75 + file.depth * 0.8}rem` }}
                >
                  <span className="truncate uppercase tracking-wide">
                    {file.is_dir ? `[DIR] ${file.path}` : file.path}
                  </span>
                  <span className="ml-3 shrink-0 text-xs text-ink-muted">
                    {file.is_dir ? "-" : formatBytes(file.size)}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p className="mt-4 text-sm text-ink-muted">No file entries available.</p>
        )}
      </section>
    </div>
  );
}
