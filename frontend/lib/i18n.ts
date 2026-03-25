import type { SiteLocale } from "@/lib/site-preferences";

export type SiteCopy = {
  layout: {
    navLatest: string;
    navTrending: string;
    navTor: string;
    navDmca: string;
    mobileHome: string;
    mobileHot: string;
    mobileDht: string;
    mobileSet: string;
    langSwitchAria: string;
    themeSwitchAria: string;
    langButtonLabel: string;
  };
  home: {
    totalIndexed: string;
    searchPlaceholder: string;
    searchButton: string;
    searchDisabled: string;
    cardFastTitle: string;
    cardFastBody: string;
    cardNoLogsTitle: string;
    cardNoLogsBody: string;
    cardP2PTitle: string;
    cardP2PBody: string;
    categories: Record<"all" | "video" | "audio" | "doc" | "app" | "other", string>;
  };
  search: {
    emptyTitle: string;
    emptyHint: string;
    failedTitle: string;
    failedHint: string;
    resultTitle: string;
    apply: string;
    noResults: string;
    summary: (total: number, tookMs: number) => string;
    sorts: Record<"relevance" | "latest" | "hot" | "size_desc" | "size_asc", string>;
  };
  latest: {
    disabled: string;
    unavailable: string;
    title: string;
    filter: string;
    noData: string;
  };
  trending: {
    disabled: string;
    unavailable: string;
    title: string;
    apply: string;
    noData: string;
  };
  torrent: {
    category: string;
    totalSize: string;
    pieceLength: string;
    fileCount: string;
    firstSeen: string;
    lastSeen: string;
    hotScore: string;
    openMagnet: string;
    files: string;
    noFiles: string;
    dirPrefix: string;
  };
  tor: {
    title: string;
    description: string;
  };
  login: {
    privateTitle: string;
    privateDesc: string;
    passwordPlaceholder: string;
    unlocking: string;
    unlock: string;
    adminTitle: string;
    adminDesc: string;
    signingIn: string;
    signIn: string;
    failed: string;
  };
  pagination: {
    prev: string;
    next: string;
    page: (current: number, total: number) => string;
  };
  torrentCard: {
    untitled: string;
    hash: string;
    size: string;
    files: string;
    hotScore: string;
    observations: string;
    firstSeen: string;
    lastSeen: string;
    magnet: string;
    details: string;
  };
  admin: {
    title: string;
    crawlerTitle: string;
    sampledEvery: (seconds: number) => string;
    logout: string;
    crawlerStatus: string;
    sampledAt: string;
    discoveredTotal: string;
    metadataSuccess: string;
    metadataFail: string;
    metadataSuccessRate: string;
    metadataQueue: string;
    nodeQueue: string;
    workerPressure: string;
    udpReceiveDropRate: string;
    siteSettingsTitle: string;
    siteSettingsDesc: string;
    saveSettings: string;
    saving: string;
    siteTitle: string;
    siteDescription: string;
    homeHeroMarkdown: string;
    updatedAt: string;
    heroPreview: string;
    saveSuccess: string;
    window: (points: number, minutes: number) => string;
    unit: (unit: string) => string;
    noChartData: string;
    prometheusDisabled: string;
  };
};

const EN: SiteCopy = {
  layout: {
    navLatest: "Latest",
    navTrending: "Trending",
    navTor: "Tor",
    navDmca: "DMCA",
    mobileHome: "Home",
    mobileHot: "Hot",
    mobileDht: "DHT",
    mobileSet: "Set",
    langSwitchAria: "Switch language",
    themeSwitchAria: "Toggle theme",
    langButtonLabel: "EN",
  },
  home: {
    totalIndexed: "Total Torrents Indexed",
    searchPlaceholder: "MAGNET HASH OR FILENAME...",
    searchButton: "Search",
    searchDisabled: "Search feature is disabled by environment flag.",
    cardFastTitle: "Fast_Indexing",
    cardFastBody:
      "Real-time DHT crawler mapping the global decentralized network. Zero-latency search for the most elusive magnet links.",
    cardNoLogsTitle: "No_Logs",
    cardNoLogsBody:
      "Privacy by design. We don't track your queries. The network is yours to explore without footprints or centralized interference.",
    cardP2PTitle: "P2P_Power",
    cardP2PBody:
      "Harnessing the strength of distributed systems. No central server dependency for data integrity or availability.",
    categories: {
      all: "All",
      video: "Video",
      audio: "Audio",
      doc: "Doc",
      app: "App",
      other: "Other",
    },
  },
  search: {
    emptyTitle: "Search",
    emptyHint: "Provide keyword or info hash in query string.",
    failedTitle: "Search failed",
    failedHint: "Backend API is unavailable.",
    resultTitle: "Search Results",
    apply: "Apply",
    noResults: "No results",
    summary: (total, tookMs) => `${total} matches · took ${tookMs}ms`,
    sorts: {
      relevance: "Relevance",
      latest: "Latest",
      hot: "Hot",
      size_desc: "Size Desc",
      size_asc: "Size Asc",
    },
  },
  latest: {
    disabled: "Latest Disabled",
    unavailable: "Latest unavailable",
    title: "Latest Torrents",
    filter: "Filter",
    noData: "No data",
  },
  trending: {
    disabled: "Trending Disabled",
    unavailable: "Trending unavailable",
    title: "Trending Torrents",
    apply: "Apply",
    noData: "No data",
  },
  torrent: {
    category: "Category",
    totalSize: "Total Size",
    pieceLength: "Piece Length",
    fileCount: "File Count",
    firstSeen: "First Seen",
    lastSeen: "Last Seen",
    hotScore: "Hot Score",
    openMagnet: "Open Magnet Link",
    files: "Files",
    noFiles: "No file entries available.",
    dirPrefix: "[DIR]",
  },
  tor: {
    title: "Tor",
    description:
      "This project currently keeps Tor as a static frontend entry by design. No backend Tor proxy/search API is enabled.",
  },
  login: {
    privateTitle: "Private Access",
    privateDesc: "This site is in private mode. Enter password to continue.",
    passwordPlaceholder: "PASSWORD",
    unlocking: "Unlocking...",
    unlock: "Unlock Site",
    adminTitle: "Admin Login",
    adminDesc: "Enter dashboard password to access crawler metrics.",
    signingIn: "Signing In...",
    signIn: "Sign In",
    failed: "Login failed",
  },
  pagination: {
    prev: "Prev",
    next: "Next",
    page: (current, total) => `Page ${current} / ${total}`,
  },
  torrentCard: {
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
  },
  admin: {
    title: "Admin Dashboard",
    crawlerTitle: "Crawler Dashboard",
    sampledEvery: (seconds) => `Full Prometheus metrics sampled every ${seconds}s.`,
    logout: "Logout",
    crawlerStatus: "Crawler Status",
    sampledAt: "Sampled At",
    discoveredTotal: "Discovered Total",
    metadataSuccess: "Metadata Success",
    metadataFail: "Metadata Fail",
    metadataSuccessRate: "Metadata Success Rate",
    metadataQueue: "Metadata Queue",
    nodeQueue: "Node Queue",
    workerPressure: "Worker Pressure",
    udpReceiveDropRate: "UDP Receive Drop Rate",
    siteSettingsTitle: "Site Settings",
    siteSettingsDesc: "Control metadata title/description and homepage hero markdown.",
    saveSettings: "Save Settings",
    saving: "Saving...",
    siteTitle: "Site Title",
    siteDescription: "Site Description",
    homeHeroMarkdown: "Home Hero Markdown",
    updatedAt: "Updated at",
    heroPreview: "Hero Preview",
    saveSuccess: "Saved. Site content updated immediately.",
    window: (points, minutes) => `Window: ${points} points · ${minutes} minutes`,
    unit: (unit) => `Unit: ${unit}`,
    noChartData: "No data yet for this chart.",
    prometheusDisabled:
      "Prometheus exporter is disabled or not initialized. Enable CRAWLER_PROMETHEUS_ENABLED=true.",
  },
};

const ZH: SiteCopy = {
  layout: {
    navLatest: "最新",
    navTrending: "热榜",
    navTor: "Tor",
    navDmca: "版权",
    mobileHome: "首页",
    mobileHot: "热度",
    mobileDht: "DHT",
    mobileSet: "设置",
    langSwitchAria: "切换语言",
    themeSwitchAria: "切换明暗主题",
    langButtonLabel: "中",
  },
  home: {
    totalIndexed: "已索引种子总量",
    searchPlaceholder: "输入磁力哈希或文件名...",
    searchButton: "搜索",
    searchDisabled: "搜索功能已被环境变量关闭。",
    cardFastTitle: "快速索引",
    cardFastBody: "实时 DHT 爬虫持续映射去中心化网络，低延迟发现和检索磁力资源。",
    cardNoLogsTitle: "隐私优先",
    cardNoLogsBody: "默认不记录你的检索行为。你可以更自由地探索网络内容与分发节点。",
    cardP2PTitle: "P2P 能力",
    cardP2PBody: "依托分布式系统能力，降低单点依赖，提升资源可用性和网络弹性。",
    categories: {
      all: "全部",
      video: "视频",
      audio: "音频",
      doc: "文档",
      app: "应用",
      other: "其他",
    },
  },
  search: {
    emptyTitle: "搜索",
    emptyHint: "请在查询参数中提供关键词或 info hash。",
    failedTitle: "搜索失败",
    failedHint: "后端 API 暂不可用。",
    resultTitle: "搜索结果",
    apply: "应用",
    noResults: "没有结果",
    summary: (total, tookMs) => `${total} 条结果 · 耗时 ${tookMs}ms`,
    sorts: {
      relevance: "相关度",
      latest: "最新",
      hot: "热度",
      size_desc: "大小降序",
      size_asc: "大小升序",
    },
  },
  latest: {
    disabled: "最新列表已禁用",
    unavailable: "最新列表暂不可用",
    title: "最新种子",
    filter: "筛选",
    noData: "暂无数据",
  },
  trending: {
    disabled: "热榜已禁用",
    unavailable: "热榜暂不可用",
    title: "热门种子",
    apply: "应用",
    noData: "暂无数据",
  },
  torrent: {
    category: "分类",
    totalSize: "总大小",
    pieceLength: "分片大小",
    fileCount: "文件数量",
    firstSeen: "首次发现",
    lastSeen: "最近发现",
    hotScore: "热度分数",
    openMagnet: "打开磁力链接",
    files: "文件列表",
    noFiles: "暂无文件条目。",
    dirPrefix: "[目录]",
  },
  tor: {
    title: "Tor",
    description: "当前版本仅保留 Tor 静态入口，不提供后端 Tor 代理或搜索 API。",
  },
  login: {
    privateTitle: "私有访问",
    privateDesc: "站点已启用私有模式，请输入密码继续访问。",
    passwordPlaceholder: "密码",
    unlocking: "解锁中...",
    unlock: "解锁站点",
    adminTitle: "管理员登录",
    adminDesc: "请输入管理面板密码以访问爬虫监控。",
    signingIn: "登录中...",
    signIn: "登录",
    failed: "登录失败",
  },
  pagination: {
    prev: "上一页",
    next: "下一页",
    page: (current, total) => `第 ${current} / ${total} 页`,
  },
  torrentCard: {
    untitled: "未命名",
    hash: "哈希",
    size: "大小",
    files: "文件数",
    hotScore: "热度",
    observations: "观测次数",
    firstSeen: "首次发现",
    lastSeen: "最近发现",
    magnet: "磁力",
    details: "详情",
  },
  admin: {
    title: "管理面板",
    crawlerTitle: "爬虫仪表板",
    sampledEvery: (seconds) => `Prometheus 指标采样间隔：${seconds} 秒。`,
    logout: "退出登录",
    crawlerStatus: "爬虫状态",
    sampledAt: "采样时间",
    discoveredTotal: "累计发现",
    metadataSuccess: "元数据成功",
    metadataFail: "元数据失败",
    metadataSuccessRate: "元数据成功率",
    metadataQueue: "元数据队列",
    nodeQueue: "节点队列",
    workerPressure: "Worker 压力",
    udpReceiveDropRate: "UDP 接收丢包率",
    siteSettingsTitle: "站点设置",
    siteSettingsDesc: "用于配置站点标题、描述和首页主标题 Markdown。",
    saveSettings: "保存设置",
    saving: "保存中...",
    siteTitle: "站点标题",
    siteDescription: "站点描述",
    homeHeroMarkdown: "首页主标题 Markdown",
    updatedAt: "更新时间",
    heroPreview: "主标题预览",
    saveSuccess: "保存成功，站点文案已即时生效。",
    window: (points, minutes) => `窗口：${points} 点 · ${minutes} 分钟`,
    unit: (unit) => `单位：${unit}`,
    noChartData: "该图表暂无数据。",
    prometheusDisabled: "Prometheus 导出器未启用或未初始化，请开启 CRAWLER_PROMETHEUS_ENABLED=true。",
  },
};

const ADMIN_TAB_TITLES_ZH: Record<string, string> = {
  overview: "总览",
  metadata: "元数据",
  network: "网络",
  protocol: "协议",
  errors: "错误",
};

const ADMIN_CHART_TITLES_ZH: Record<string, string> = {
  overview_discovery_rate: "InfoHash 发现速率",
  overview_metadata_outcome_rate: "元数据结果速率",
  overview_queue_and_pressure: "队列与压力",
  metadata_fetch_attempts_rate: "元数据抓取尝试",
  metadata_fail_reason_rate: "按原因统计的元数据失败",
  metadata_connection_result_rate: "连接结果",
  metadata_handshake_result_rate: "握手结果",
  metadata_download_throughput: "元数据下载吞吐",
  metadata_size_distribution: "元数据大小 P50/P95/AVG",
  network_udp_packets_received: "UDP 接收包（按状态）",
  network_udp_packets_sent: "UDP 发送包（按类型）",
  network_udp_bytes: "UDP 字节吞吐",
  network_nodes_discovered: "发现节点（按 IP 版本）",
  protocol_queries: "DHT 查询（按类型）",
  protocol_messages: "DHT 消息处理量",
  protocol_parse_errors: "消息解析错误",
  errors_metadata_fail: "元数据失败原因",
  errors_announce_peer_blocked: "Announce Peer 拦截",
  errors_udp_receive_status: "UDP 接收状态",
};

const CATEGORY_LABELS_ZH: Record<string, string> = {
  all: "全部",
  video: "视频",
  audio: "音频",
  doc: "文档",
  app: "应用",
  other: "其他",
};

export function getCopy(locale: SiteLocale): SiteCopy {
  return locale === "zh" ? ZH : EN;
}

export function localizeCategoryLabel(locale: SiteLocale, key: string, fallback: string): string {
  if (locale !== "zh") {
    return fallback;
  }

  return CATEGORY_LABELS_ZH[key] ?? fallback;
}

export function localizeAdminTabTitle(
  locale: SiteLocale,
  tabId: string,
  fallback: string,
): string {
  if (locale !== "zh") {
    return fallback;
  }

  return ADMIN_TAB_TITLES_ZH[tabId] ?? fallback;
}

export function localizeAdminChartTitle(
  locale: SiteLocale,
  chartId: string,
  fallback: string,
): string {
  if (locale !== "zh") {
    return fallback;
  }

  return ADMIN_CHART_TITLES_ZH[chartId] ?? fallback;
}
