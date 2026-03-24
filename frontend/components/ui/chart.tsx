"use client";

import * as React from "react";
import { Tooltip as RechartsTooltip, type TooltipProps } from "recharts";

export type ChartConfig = Record<
  string,
  {
    label: string;
    color: string;
  }
>;

type ChartContextValue = {
  config: ChartConfig;
};

const ChartContext = React.createContext<ChartContextValue | null>(null);

function useChartContext() {
  const context = React.useContext(ChartContext);
  if (!context) {
    throw new Error("useChartContext must be used inside ChartContainer");
  }

  return context;
}

type ChartContainerProps = React.HTMLAttributes<HTMLDivElement> & {
  config: ChartConfig;
};

export function ChartContainer({ config, className, style, ...props }: ChartContainerProps) {
  const chartStyle = {
    ...style,
    ...Object.fromEntries(
      Object.entries(config).map(([key, value]) => [`--color-${key}`, value.color]),
    ),
  } as React.CSSProperties;

  return (
    <ChartContext.Provider value={{ config }}>
      <div className={className} style={chartStyle} {...props} />
    </ChartContext.Provider>
  );
}

export const ChartTooltip = RechartsTooltip;

type TooltipPayloadItem = {
  name?: string;
  value?: number | string;
  dataKey?: string;
  color?: string;
};

type ChartTooltipContentProps = {
  active?: boolean;
  payload?: TooltipPayloadItem[];
  label?: string | number;
  hideLabel?: boolean;
  className?: string;
};

export function ChartTooltipContent({
  active,
  payload,
  label,
  hideLabel = false,
  className,
}: ChartTooltipContentProps) {
  const { config } = useChartContext();

  if (!active || !payload?.length) {
    return null;
  }

  return (
    <div className={className ?? "border-2 border-ink bg-paper p-3 text-xs shadow-hard-sm"}>
      {!hideLabel ? <div className="mb-2 font-semibold uppercase">{label}</div> : null}
      <div className="space-y-1">
        {payload.map((item, index) => {
          const key = String(item.dataKey ?? item.name ?? index);
          const itemConfig = config[key];
          const dotColor = item.color ?? itemConfig?.color ?? "#111";
          const displayLabel = itemConfig?.label ?? String(item.name ?? key);
          const value = typeof item.value === "number" ? item.value.toFixed(2) : String(item.value ?? "-");

          return (
            <div key={key} className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2">
                <span className="inline-block h-2.5 w-2.5" style={{ backgroundColor: dotColor }} />
                <span>{displayLabel}</span>
              </div>
              <span className="tabular-nums">{value}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export type RechartsTooltipTypedProps = TooltipProps<number, string>;
