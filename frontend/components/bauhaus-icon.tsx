import type { LucideProps } from "lucide-react";
import type { ComponentType } from "react";
import {
  Database,
  Home,
  Moon,
  Search,
  Settings,
  TrendingUp,
} from "lucide-react";

type BauhausIconName =
  | "dark_mode"
  | "search"
  | "home"
  | "trending_up"
  | "database"
  | "settings";

const ICONS: Record<BauhausIconName, ComponentType<LucideProps>> = {
  dark_mode: Moon,
  search: Search,
  home: Home,
  trending_up: TrendingUp,
  database: Database,
  settings: Settings,
};

export function BauhausIcon({
  name,
  className,
  strokeWidth = 2.2,
}: {
  name: BauhausIconName;
  className?: string;
  strokeWidth?: number;
}) {
  const Icon = ICONS[name];
  return <Icon className={className} strokeWidth={strokeWidth} aria-hidden="true" />;
}
