export type DateFilter = "all" | "today" | "week" | "month" | "custom";

export interface Screenshot {
  filename: string;
  path: string;
  size: number;
  takenAt: number;
}

export interface World {
  folderName: string;
  name: string;
  path: string;
  lastPlayed: number | null;
  gameMode: string | null;
  cheatsEnabled: boolean;
  versionName: string | null;
  iconBase64: string | null;
  size: number;
}

export interface Server {
  name: string;
  ip: string;
  iconBase64: string | null;
  hidden: boolean;
  acceptTextures: boolean;
}

export interface ScreenshotsResponse {
  screenshots: Screenshot[];
}

export interface WorldsResponse {
  worlds: World[];
}

export interface ServersResponse {
  servers: Server[];
}

export interface InstanceDetail {
  totalPlayTime: number;
  recentScreenshots: Screenshot[];
  recentWorlds: World[];
  savedServers: Server[];
}
