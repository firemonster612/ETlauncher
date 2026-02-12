export type TaskType =
	| 'gameDownload'
	| 'contentInstall'
	| 'modpackInstall'
	| 'instanceUpdate'
	| 'instanceSetup'
	| 'loaderInstall'
	| 'launcherUpdate'
	| 'versionMigration'
	| 'instanceImport'
	| 'modpackImport'
	| 'instanceExport'
	| 'contentScan';

export type TaskStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface TaskProgress {
	current: number;
	total: number;
	/** Override percentage (0-100). When set, use this instead of current/total. */
	percent?: number;
	speedBytesPerSec?: number;
	currentItem?: string;
	stage?: string;
}

export interface TrackedTaskInfo {
	id: string;
	taskType: TaskType;
	label: string;
	status: TaskStatus;
	progress?: TaskProgress;
	instanceId?: string;
	error?: string;
	createdAt: number;
}
