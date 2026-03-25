import Link from "next/link";

import type { SiteCopy } from "@/lib/i18n";

type Props = {
  pathname: string;
  page: number;
  pageSize: number;
  total: number;
  query: Record<string, string | number | undefined>;
  labels?: SiteCopy["pagination"];
};

function buildHref(
  pathname: string,
  query: Record<string, string | number | undefined>,
  page: number,
  pageSize: number,
): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === "") continue;
    params.set(key, String(value));
  }
  params.set("page", String(page));
  params.set("page_size", String(pageSize));

  return `${pathname}?${params.toString()}`;
}

export function Pagination({ pathname, page, pageSize, total, query, labels }: Props) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const previousPage = Math.max(1, page - 1);
  const nextPage = Math.min(totalPages, page + 1);
  const prevLabel = labels?.prev ?? "Prev";
  const nextLabel = labels?.next ?? "Next";
  const pageLabel = labels?.page(page, totalPages) ?? `Page ${page} / ${totalPages}`;

  return (
    <nav className="mt-8 flex items-center justify-between border-4 border-ink bg-paper p-4 shadow-hard-sm">
      <Link
        href={buildHref(pathname, query, previousPage, pageSize)}
        aria-disabled={page <= 1}
        className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-paper px-3 py-1 text-sm font-bold uppercase tracking-wide transition-all hover:bg-accent-yellow aria-disabled:pointer-events-none aria-disabled:opacity-50"
      >
        {prevLabel}
      </Link>
      <p className="text-sm font-semibold uppercase tracking-wide text-ink-muted">
        {pageLabel}
      </p>
      <Link
        href={buildHref(pathname, query, nextPage, pageSize)}
        aria-disabled={page >= totalPages}
        className="bauhaus-shadow-sm bauhaus-press inline-flex items-center justify-center border-2 border-ink bg-paper px-3 py-1 text-sm font-bold uppercase tracking-wide transition-all hover:bg-accent-yellow aria-disabled:pointer-events-none aria-disabled:opacity-50"
      >
        {nextLabel}
      </Link>
    </nav>
  );
}
