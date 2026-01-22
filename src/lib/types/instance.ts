/** Type of mod loader for an instance */
export type LoaderType =
	| 'vanilla'
	| 'forge'
	| 'neoforge'
	| 'fabric'
	| 'quilt'
	| 'liteloader'
	| 'unknown';

/** Platform that hosts modpacks */
export type ModpackPlatform = 'modrinth' | 'curseforge' | 'ftb' | 'technic' | 'atlauncher';

/** A Minecraft instance configuration */
export interface Instance {
	/** Unique identifier (UUID) */
	id: string;
	/** Display name */
	name: string;
	/** Minecraft version (e.g., "1.21.4") */
	minecraftVersion: string;
	/** Mod loader type */
	loaderType: LoaderType;
	/** Mod loader version (if applicable) */
	loaderVersion?: string;
	/** Unix timestamp when instance was created */
	createdAt: number;
	/** Unix timestamp when instance was last played */
	lastPlayedAt?: number;
	/** Total play time in seconds */
	totalPlayTime: number;
	/** Path to custom icon (relative to instance folder) */
	iconPath?: string;
	/** Override: path to Java executable */
	javaPath?: string;
	/** Override: minimum memory allocation (MB) */
	memoryMinMb?: number;
	/** Override: maximum memory allocation (MB) */
	memoryMaxMb?: number;
	/** Override: additional JVM arguments */
	jvmArgs?: string;
	/** Override: additional game arguments */
	gameArgs?: string;
	/** Override: game window width */
	resolutionWidth?: number;
	/** Override: game window height */
	resolutionHeight?: number;
	/** Modpack platform (if created from a modpack) */
	modpackPlatform?: ModpackPlatform;
	/** Modpack ID on the platform */
	modpackId?: string;
	/** Installed modpack version ID */
	modpackVersionId?: string;
}

/** Request to create a new instance */
export interface CreateInstanceRequest {
	name: string;
	minecraftVersion: string;
	loaderType?: LoaderType;
	loaderVersion?: string;
}

/** Request to update an existing instance */
export interface UpdateInstanceRequest {
	name?: string;
	loaderType?: LoaderType;
	loaderVersion?: string;
	iconPath?: string;
	javaPath?: string;
	memoryMinMb?: number;
	memoryMaxMb?: number;
	jvmArgs?: string;
	gameArgs?: string;
	resolutionWidth?: number;
	resolutionHeight?: number;
}

/** Download progress information */
export interface DownloadProgress {
	totalFiles: number;
	completedFiles: number;
	currentFile: string;
	totalBytes: number;
	downloadedBytes: number;
	speedBytesPerSec: number;
}

/** Instance launch status */
export type LaunchStatus =
	| { status: 'idle' }
	| { status: 'checkingAccount' }
	| { status: 'refreshingToken' }
	| { status: 'loadingVersion' }
	| { status: 'verifyingFiles'; checked: number; total: number }
	| { status: 'downloading'; progress: DownloadProgress }
	| { status: 'checkingJava'; version: number }
	| { status: 'downloadingJava'; version: number; progress: number }
	| { status: 'buildingClasspath' }
	| { status: 'launching' }
	| { status: 'running'; pid: number }
	| { status: 'windowReady'; pid: number }
	| { status: 'stopped'; exitCode: number }
	| { status: 'crashed'; message: string };

/** Instance setup status (used during instance creation) */
export type InstanceSetupStatus =
	| { status: 'pending' }
	| { status: 'preparing'; message: string }
	| { status: 'downloadingGameFiles'; progress: DownloadProgress }
	| { status: 'installingLoader'; stage: string; progress: number }
	| { status: 'complete' }
	| { status: 'failed'; message: string };
