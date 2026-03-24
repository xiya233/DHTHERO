"use client";

import { useEffect, useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

import {
  ChartContainer,
  type ChartConfig,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";

type AdminDashboardPoint = {
  timestamp: string;
  value: number;
};

type AdminDashboardSeries = {
  id: string;
  label: string;
  labels: Record<string, string>;
  points: AdminDashboardPoint[];
};

type AdminDashboardChart = {
  id: string;
  title: string;
  unit: string;
  render: "lines" | "stacked";
  series: AdminDashboardSeries[];
};

type AdminDashboardTab = {
  id: string;
  title: string;
  charts: AdminDashboardChart[];
};

type AdminDashboardResponse = {
  now: {
    crawler_status: string;
    info_hash_discovered_total: number;
    metadata_fetch_success_total: number;
    metadata_fetch_fail_total: number;
    metadata_success_rate: number;
    metadata_queue_size: number;
    node_queue_size: number;
    metadata_worker_pressure: number;
    udp_packets_received_total: number;
    udp_packets_sent_total: number;
    udp_receive_drop_rate: number;
    updated_at: string;
  };
  window: {
    sample_interval_secs: number;
    points: number;
    horizon_secs: number;
    prometheus_enabled: boolean;
  };
  tabs: AdminDashboardTab[];
};

type ChartRow = {
  timestamp: string;
  timeLabel: string;
} & Record<string, number | string>;

const CHART_COLORS = [
  "#ffcc00",
  "#0055ff",
  "#e63b2e",
  "#198754",
  "#f97316",
  "#6f42c1",
  "#0ea5e9",
  "#111827",
  "#b45309",
  "#ef4444",
];

function formatCompact(value: number): string {
  if (!Number.isFinite(value)) return "-";
  if (Math.abs(value) >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toFixed(0);
}

function formatPercent(value: number): string {
  if (!Number.isFinite(value)) return "-";
  return `${(value * 100).toFixed(1)}%`;
}

function formatTimestamp(value?: string): string {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleString("en-US", {
    hour12: false,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function chartColor(index: number): string {
  return CHART_COLORS[index % CHART_COLORS.length];
}

function buildChartConfig(chart: AdminDashboardChart): ChartConfig {
  return Object.fromEntries(
    chart.series.map((series, index) => [
      series.id,
      {
        label: series.label,
        color: chartColor(index),
      },
    ]),
  );
}

function buildChartRows(chart: AdminDashboardChart): ChartRow[] {
  const byTimestamp = new Map<string, ChartRow>();

  for (const series of chart.series) {
    for (const point of series.points) {
      if (!byTimestamp.has(point.timestamp)) {
        byTimestamp.set(point.timestamp, {
          timestamp: point.timestamp,
          timeLabel: new Date(point.timestamp).toLocaleTimeString("en-US", {
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
            hour12: false,
          }),
        });
      }

      const row = byTimestamp.get(point.timestamp);
      if (row) {
        row[series.id] = point.value;
      }
    }
  }

  return Array.from(byTimestamp.values()).sort((a, b) =>
    String(a.timestamp).localeCompare(String(b.timestamp)),
  );
}

export default function AdminDashboardPage() {
  const [data, setData] = useState<AdminDashboardResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mounted, setMounted] = useState(false);
  const [activeTabId, setActiveTabId] = useState<string>("overview");

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    let stopped = false;

    const load = async () => {
      try {
        const response = await fetch("/api/admin/dashboard", {
          cache: "no-store",
        });

        if (response.status === 401) {
          window.location.href = "/admin/login";
          return;
        }

        if (!response.ok) {
          const payload = (await response.json().catch(() => null)) as
            | { message?: string }
            | null;
          throw new Error(payload?.message || `dashboard request failed: ${response.status}`);
        }

        const payload = (await response.json()) as AdminDashboardResponse;
        if (!stopped) {
          setData(payload);
          setError(null);
        }
      } catch (err) {
        if (!stopped) {
          const message = err instanceof Error ? err.message : "failed to load dashboard";
          setError(message);
        }
      }
    };

    void load();
    const timer = window.setInterval(() => {
      void load();
    }, 5_000);

    return () => {
      stopped = true;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    const tabs = data?.tabs ?? [];
    if (!tabs.length) {
      return;
    }

    if (!tabs.some((tab) => tab.id === activeTabId)) {
      setActiveTabId(tabs[0]?.id ?? "overview");
    }
  }, [activeTabId, data?.tabs]);

  const activeTab = useMemo(() => {
    const tabs = data?.tabs ?? [];
    return tabs.find((tab) => tab.id === activeTabId) ?? tabs[0] ?? null;
  }, [activeTabId, data?.tabs]);

  async function logout() {
    await fetch("/api/admin/logout", { method: "POST" });
    window.location.href = "/admin/login";
  }

  if (!data && error) {
    return (
      <section className="w-full max-w-3xl border-4 border-ink bg-paper p-8 shadow-hard-sm">
        <h1 className="font-headline text-4xl font-black uppercase">Admin Dashboard</h1>
        <p className="mt-4 text-sm text-accent-red">{error}</p>
      </section>
    );
  }

  return (
    <div className="w-full space-y-8">
      <section className="border-4 border-ink bg-paper p-6 shadow-hard-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h1 className="font-headline text-4xl font-black uppercase">Crawler Dashboard</h1>
            <p className="mt-2 text-sm text-ink-muted">
              Full Prometheus metrics sampled every {data?.window.sample_interval_secs ?? 0}s.
            </p>
          </div>
          <button
            onClick={logout}
            className="bauhaus-shadow-sm bauhaus-press border-2 border-ink bg-paper px-4 py-2 font-headline text-sm font-bold uppercase transition-all hover:bg-accent-yellow"
          >
            Logout
          </button>
        </div>

        {error ? <p className="mt-4 text-sm text-accent-red">{error}</p> : null}

        <div className="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-5">
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Crawler Status</p>
            <p className="font-headline text-2xl font-black uppercase">{data?.now.crawler_status ?? "-"}</p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Sampled At</p>
            <p className="font-headline text-sm font-black uppercase">{formatTimestamp(data?.now.updated_at)}</p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Discovered Total</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.info_hash_discovered_total ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Metadata Success</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.metadata_fetch_success_total ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Metadata Fail</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.metadata_fetch_fail_total ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Metadata Success Rate</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatPercent(data?.now.metadata_success_rate ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Metadata Queue</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.metadata_queue_size ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Node Queue</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.node_queue_size ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Worker Pressure</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatPercent(data?.now.metadata_worker_pressure ?? 0)}
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">UDP Receive Drop Rate</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatPercent(data?.now.udp_receive_drop_rate ?? 0)}
            </p>
          </article>
        </div>
      </section>

      <section className="space-y-5 border-4 border-ink bg-paper p-5 shadow-hard-sm">
        <div className="flex flex-wrap items-center gap-3">
          {(data?.tabs ?? []).map((tab) => {
            const active = tab.id === activeTab?.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTabId(tab.id)}
                className={`bauhaus-shadow-sm bauhaus-press border-2 border-ink px-4 py-2 font-headline text-sm font-bold uppercase transition-all ${
                  active
                    ? "bg-accent-yellow text-ink"
                    : "bg-paper text-ink hover:bg-ink hover:text-paper"
                }`}
              >
                {tab.title}
              </button>
            );
          })}
        </div>

        <p className="text-xs uppercase text-ink-muted">
          Window: {data?.window.points ?? 0} points · {Math.round((data?.window.horizon_secs ?? 0) / 60)} minutes
        </p>

        <div className="grid gap-5 xl:grid-cols-2">
          {(activeTab?.charts ?? []).map((chart) => {
            const config = buildChartConfig(chart);
            const rows = buildChartRows(chart);
            const isStacked = chart.render === "stacked";

            return (
              <article key={chart.id} className="border-2 border-ink bg-paper p-4">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h2 className="font-headline text-xl font-black uppercase">{chart.title}</h2>
                  <span className="text-xs uppercase text-ink-muted">Unit: {chart.unit}</span>
                </div>

                {mounted ? (
                  chart.series.length && rows.length ? (
                    <ChartContainer config={config} className="mt-4 h-72 w-full">
                      <ResponsiveContainer width="100%" height="100%">
                        {isStacked ? (
                          <AreaChart data={rows}>
                            <CartesianGrid strokeDasharray="3 3" stroke="#c4bdb3" />
                            <XAxis dataKey="timeLabel" minTickGap={24} tick={{ fontSize: 11 }} />
                            <YAxis tick={{ fontSize: 11 }} />
                            <ChartTooltip content={<ChartTooltipContent />} />
                            <Legend wrapperStyle={{ fontSize: 11 }} />
                            {chart.series.map((series) => (
                              <Area
                                key={series.id}
                                type="monotone"
                                dataKey={series.id}
                                name={series.label}
                                stackId="stack"
                                stroke={`var(--color-${series.id})`}
                                fill={`var(--color-${series.id})`}
                                fillOpacity={0.22}
                                strokeWidth={2}
                                dot={false}
                              />
                            ))}
                          </AreaChart>
                        ) : (
                          <LineChart data={rows}>
                            <CartesianGrid strokeDasharray="3 3" stroke="#c4bdb3" />
                            <XAxis dataKey="timeLabel" minTickGap={24} tick={{ fontSize: 11 }} />
                            <YAxis tick={{ fontSize: 11 }} />
                            <ChartTooltip content={<ChartTooltipContent />} />
                            <Legend wrapperStyle={{ fontSize: 11 }} />
                            {chart.series.map((series) => (
                              <Line
                                key={series.id}
                                type="monotone"
                                dataKey={series.id}
                                name={series.label}
                                stroke={`var(--color-${series.id})`}
                                dot={false}
                                strokeWidth={2}
                              />
                            ))}
                          </LineChart>
                        )}
                      </ResponsiveContainer>
                    </ChartContainer>
                  ) : (
                    <div className="mt-4 flex h-72 items-center justify-center border border-dashed border-ink/40 text-sm text-ink-muted">
                      No data yet for this chart.
                    </div>
                  )
                ) : null}
              </article>
            );
          })}
        </div>
      </section>

      {!data?.window.prometheus_enabled ? (
        <section className="border-4 border-ink bg-paper p-5 text-sm text-accent-red shadow-hard-sm">
          Prometheus exporter is disabled or not initialized. Enable `CRAWLER_PROMETHEUS_ENABLED=true`.
        </section>
      ) : null}
    </div>
  );
}
