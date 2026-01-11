// Re-export all types
export * from './account';
export * from './content';
export * from './instance';
export * from './instance-detail';
export * from './loader';
export * from './minecraft';
export * from './modpack';
export * from './settings';
export * from './update';

/** Error returned from Tauri commands */
export interface CommandError {
	code: string;
	message: string;
}
