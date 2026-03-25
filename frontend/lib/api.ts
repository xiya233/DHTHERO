import { getPrivateSitePasswordFromEnv, isPrivateModeActiveFromEnv } from "@/lib/site-auth";

export type FeatureFlags = {
  search_enabled: boolean;
  latest_enabled: boolean;
  trending_enabled: boolean;
  file_tree_enabled: boolean;
  audit_enabled: boolean;
};

export type SiteStats = {
  total_torrents_indexed: number;
  total_size_bytes: number;
  last_crawl_at: string | null;
  crawler_status: string;
};

export type SiteContent = {
  site_title: string;
  site_description: string;
  home_hero_markdown: string;
  updated_at: string | null;
};

export type CategoryItem = {
  key: string;
  label: string;
  count: number;
};

export type CategoriesResponse = {
  categories: CategoryItem[];
};

export type TorrentListItem = {
  info_hash: string;
  name: string;
  magnet_link: string;
  category: string;
  total_size: number;
  file_count: number;
  first_seen_at: string;
  last_seen_at: string;
  trend_score: number;
  observations: number;
};

export type ListResponse<T> = {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  took_ms: number;
};

export type TorrentFileEntry = {
  path: string;
  size: number;
  depth: number;
  is_dir: boolean;
};

export type TorrentDetail = {
  info_hash: string;
  name: string;
  magnet_link: string;
  category: string;
  total_size: number;
  piece_length: number;
  file_count: number;
  first_seen_at: string;
  last_seen_at: string;
  hot_score: number;
  files_preview: TorrentFileEntry[];
};

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL?.replace(/\/$/, "") ||
  "http://localhost:8080";

export const DEFAULT_SITE_CONTENT: SiteContent = {
  site_title: "DHT_MAGNET",
  site_description: "Bauhaus inspired DHT magnet search engine",
  home_hero_markdown: "SEARCH\nTHE_NET",
  updated_at: null,
};

async function apiFetch<T>(path: string): Promise<T> {
  const headers: Record<string, string> = {
    Accept: "application/json",
  };

  if (isPrivateModeActiveFromEnv()) {
    headers["x-site-password"] = getPrivateSitePasswordFromEnv();
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    cache: "no-store",
    headers,
  });

  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }

  return (await response.json()) as T;
}

function withQuery(
  path: string,
  params: Record<string, string | number | boolean | undefined>,
): string {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === "") continue;
    query.set(key, String(value));
  }
  const queryString = query.toString();
  if (!queryString) return path;
  return `${path}?${queryString}`;
}

export async function getFeatures(): Promise<FeatureFlags> {
  try {
    return await apiFetch<FeatureFlags>("/api/v1/features");
  } catch {
    return {
      search_enabled: true,
      latest_enabled: true,
      trending_enabled: true,
      file_tree_enabled: true,
      audit_enabled: true,
    };
  }
}

export async function getSiteStats(): Promise<SiteStats | null> {
  try {
    return await apiFetch<SiteStats>("/api/v1/site/stats");
  } catch {
    return null;
  }
}

export async function getSiteContent(): Promise<SiteContent> {
  try {
    return await apiFetch<SiteContent>("/api/v1/site/content");
  } catch {
    return DEFAULT_SITE_CONTENT;
  }
}

export async function getCategories(): Promise<CategoryItem[]> {
  try {
    const data = await apiFetch<CategoriesResponse>("/api/v1/categories");
    return data.categories;
  } catch {
    return [
      { key: "all", label: "All", count: 0 },
      { key: "video", label: "Video", count: 0 },
      { key: "audio", label: "Audio", count: 0 },
      { key: "doc", label: "Doc", count: 0 },
      { key: "app", label: "App", count: 0 },
      { key: "other", label: "Other", count: 0 },
    ];
  }
}

export async function searchTorrents(params: {
  q: string;
  category?: string;
  sort?: string;
  page?: number;
  pageSize?: number;
}): Promise<ListResponse<TorrentListItem>> {
  return apiFetch<ListResponse<TorrentListItem>>(
    withQuery("/api/v1/search", {
      q: params.q,
      category: params.category,
      sort: params.sort,
      page: params.page,
      page_size: params.pageSize,
    }),
  );
}

export async function getLatest(params: {
  category?: string;
  page?: number;
  pageSize?: number;
}): Promise<ListResponse<TorrentListItem>> {
  return apiFetch<ListResponse<TorrentListItem>>(
    withQuery("/api/v1/latest", {
      category: params.category,
      page: params.page,
      page_size: params.pageSize,
    }),
  );
}

export async function getTrending(params: {
  window?: string;
  category?: string;
  page?: number;
  pageSize?: number;
}): Promise<ListResponse<TorrentListItem>> {
  return apiFetch<ListResponse<TorrentListItem>>(
    withQuery("/api/v1/trending", {
      window: params.window,
      category: params.category,
      page: params.page,
      page_size: params.pageSize,
    }),
  );
}

export async function getTorrentDetail(
  infoHash: string,
): Promise<TorrentDetail | null> {
  try {
    return await apiFetch<TorrentDetail>(`/api/v1/torrents/${infoHash}`);
  } catch {
    return null;
  }
}

export async function getTorrentFiles(
  infoHash: string,
  flat = false,
): Promise<TorrentFileEntry[]> {
  try {
    const response = await apiFetch<{ files: TorrentFileEntry[] }>(
      withQuery(`/api/v1/torrents/${infoHash}/files`, { flat }),
    );
    return response.files;
  } catch {
    return [];
  }
}
