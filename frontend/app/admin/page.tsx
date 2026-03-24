"use client";

import { useEffect, useMemo, useState } from "react";
import {
  CartesianGrid,
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

type AdminDashboardNow = {
  crawler_status: string;
  info_hash_discovered_total: number;
  metadata_fetch_success_total: number;
  metadata_fetch_fail_total: number;
  metadata_queue_size: number;
  node_queue_size: number;
  metadata_worker_pressure: number;
  udp_packets_received_total: number;
  udp_packets_sent_total: number;
  updated_at: string;
};

type AdminDashboardSeriesPoint = {
  timestamp: string;
  info_hash_discovered_total: number;
  metadata_fetch_success_total: number;
  metadata_fetch_fail_total: number;
  metadata_queue_size: number;
  node_queue_size: number;
  metadata_worker_pressure: number;
  udp_packets_received_rate: number;
  udp_packets_sent_rate: number;
};

type AdminDashboardResponse = {
  now: AdminDashboardNow;
  series: AdminDashboardSeriesPoint[];
  window: {
    sample_interval_secs: number;
    points: number;
    horizon_secs: number;
    prometheus_enabled: boolean;
  };
};

const DISCOVERY_CHART_CONFIG: ChartConfig = {
  info_hash_discovered_total: { label: "InfoHash Discovered", color: "#ffcc00" },
  metadata_fetch_success_total: { label: "Metadata Success", color: "#0055ff" },
  metadata_fetch_fail_total: { label: "Metadata Fail", color: "#e63b2e" },
};

const QUEUE_CHART_CONFIG: ChartConfig = {
  metadata_queue_size: { label: "Metadata Queue", color: "#ffcc00" },
  node_queue_size: { label: "Node Queue", color: "#0055ff" },
  metadata_worker_pressure: { label: "Worker Pressure", color: "#e63b2e" },
};

const UDP_CHART_CONFIG: ChartConfig = {
  udp_packets_received_rate: { label: "UDP RX/s", color: "#0055ff" },
  udp_packets_sent_rate: { label: "UDP TX/s", color: "#ffcc00" },
};

function formatCompact(value: number): string {
  if (!Number.isFinite(value)) return "-";
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toFixed(0);
}

function toChartData(series: AdminDashboardSeriesPoint[]) {
  return series.map((point) => ({
    ...point,
    timeLabel: new Date(point.timestamp).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    }),
  }));
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

export default function AdminDashboardPage() {
  const [data, setData] = useState<AdminDashboardResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mounted, setMounted] = useState(false);

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

  const chartData = useMemo(() => toChartData(data?.series ?? []), [data?.series]);

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
              Prometheus live metrics sampled every {data?.window.sample_interval_secs ?? 0}s.
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

        <div className="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Crawler Status</p>
            <p className="font-headline text-2xl font-black uppercase">{data?.now.crawler_status ?? "-"}</p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Sampled At</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatTimestamp(data?.now.updated_at)}
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
            <p className="text-xs uppercase text-ink-muted">Worker Pressure</p>
            <p className="font-headline text-2xl font-black uppercase">
              {((data?.now.metadata_worker_pressure ?? 0) * 100).toFixed(1)}%
            </p>
          </article>
          <article className="border-2 border-ink bg-paper-soft p-3">
            <p className="text-xs uppercase text-ink-muted">Metadata Queue</p>
            <p className="font-headline text-2xl font-black uppercase">
              {formatCompact(data?.now.metadata_queue_size ?? 0)}
            </p>
          </article>
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-2">
        <article className="border-4 border-ink bg-paper p-5 shadow-hard-sm xl:col-span-2">
          <h2 className="font-headline text-2xl font-black uppercase">Discovery & Metadata</h2>
          <ChartContainer config={DISCOVERY_CHART_CONFIG} className="mt-4 h-72 w-full">
            {mounted ? (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#c4bdb3" />
                  <XAxis dataKey="timeLabel" minTickGap={24} tick={{ fontSize: 11 }} />
                  <YAxis tick={{ fontSize: 11 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="info_hash_discovered_total" stroke="var(--color-info_hash_discovered_total)" dot={false} strokeWidth={2} />
                  <Line type="monotone" dataKey="metadata_fetch_success_total" stroke="var(--color-metadata_fetch_success_total)" dot={false} strokeWidth={2} />
                  <Line type="monotone" dataKey="metadata_fetch_fail_total" stroke="var(--color-metadata_fetch_fail_total)" dot={false} strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            ) : null}
          </ChartContainer>
        </article>

        <article className="border-4 border-ink bg-paper p-5 shadow-hard-sm">
          <h2 className="font-headline text-2xl font-black uppercase">Queue Pressure</h2>
          <ChartContainer config={QUEUE_CHART_CONFIG} className="mt-4 h-72 w-full">
            {mounted ? (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#c4bdb3" />
                  <XAxis dataKey="timeLabel" minTickGap={24} tick={{ fontSize: 11 }} />
                  <YAxis tick={{ fontSize: 11 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="metadata_queue_size" stroke="var(--color-metadata_queue_size)" dot={false} strokeWidth={2} />
                  <Line type="monotone" dataKey="node_queue_size" stroke="var(--color-node_queue_size)" dot={false} strokeWidth={2} />
                  <Line type="monotone" dataKey="metadata_worker_pressure" stroke="var(--color-metadata_worker_pressure)" dot={false} strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            ) : null}
          </ChartContainer>
        </article>

        <article className="border-4 border-ink bg-paper p-5 shadow-hard-sm">
          <h2 className="font-headline text-2xl font-black uppercase">UDP Throughput</h2>
          <ChartContainer config={UDP_CHART_CONFIG} className="mt-4 h-72 w-full">
            {mounted ? (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#c4bdb3" />
                  <XAxis dataKey="timeLabel" minTickGap={24} tick={{ fontSize: 11 }} />
                  <YAxis tick={{ fontSize: 11 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="udp_packets_received_rate" stroke="var(--color-udp_packets_received_rate)" dot={false} strokeWidth={2} />
                  <Line type="monotone" dataKey="udp_packets_sent_rate" stroke="var(--color-udp_packets_sent_rate)" dot={false} strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            ) : null}
          </ChartContainer>
        </article>
      </section>

      {!data?.window.prometheus_enabled ? (
        <section className="border-4 border-ink bg-paper p-5 text-sm text-accent-red shadow-hard-sm">
          Prometheus exporter is disabled or not initialized. Enable `CRAWLER_PROMETHEUS_ENABLED=true`.
        </section>
      ) : null}
    </div>
  );
}
