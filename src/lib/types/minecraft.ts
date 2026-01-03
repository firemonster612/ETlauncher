/** Version manifest from Mojang API */
export interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: VersionEntry[];
}

/** A single version entry in the manifest */
export interface VersionEntry {
  id: string;
  type: "release" | "snapshot" | "old_beta" | "old_alpha";
  url: string;
  time: string;
  releaseTime: string;
  sha1?: string;
  complianceLevel?: number;
}

/** Detailed version information from version JSON */
export interface VersionInfo {
  id: string;
  mainClass: string;
  /** Legacy format (pre-1.13) */
  minecraftArguments?: string;
  /** Modern format (1.13+) */
  arguments?: {
    game: ArgumentValue[];
    jvm: ArgumentValue[];
  };
  libraries: Library[];
  assetIndex: AssetIndexRef;
  downloads: {
    client: DownloadInfo;
    server?: DownloadInfo;
  };
  javaVersion?: {
    component: string;
    majorVersion: number;
  };
  type?: string;
  assets?: string;
  inheritsFrom?: string;
}

export type ArgumentValue =
  | string
  | {
      rules: Rule[];
      value: string | string[];
    };

export interface Rule {
  action: "allow" | "disallow";
  os?: {
    name?: string;
    version?: string;
    arch?: string;
  };
  features?: Record<string, boolean>;
}

export interface Library {
  name: string;
  downloads?: {
    artifact?: Artifact;
    classifiers?: Record<string, Artifact>;
  };
  natives?: Record<string, string>;
  rules?: Rule[];
  extract?: {
    exclude: string[];
  };
  url?: string;
}

export interface Artifact {
  path: string;
  url: string;
  sha1: string;
  size: number;
}

export interface AssetIndexRef {
  id: string;
  sha1: string;
  size: number;
  totalSize: number;
  url: string;
}

export interface DownloadInfo {
  sha1: string;
  size: number;
  url: string;
}

/** Game log line for streaming to frontend */
export interface GameLogLine {
  instanceId: string;
  line: string;
  level: LogLevel;
  timestamp: number;
}

export type LogLevel = "info" | "warn" | "error" | "debug";
