import type { LoaderType } from './instance';

/** Type of import source */
export type ImportSourceType = 'vanillaMinecraft' | 'multiMC' | 'prismLauncher' | 'curseForgeZip';

/** Analysis result from examining an import source */
export interface ImportAnalysis {
	/** Detected Minecraft version */
	minecraftVersion?: string;
	/** Detected loader type */
	loaderType: LoaderType;
	/** Detected loader version */
	loaderVersion?: string;
	/** Number of mods found */
	modCount: number;
	/** Whether resourcepacks folder has content */
	hasResourcepacks: boolean;
	/** Whether shaderpacks folder has content */
	hasShaderpacks: boolean;
	/** Whether config folder has content */
	hasConfig: boolean;
	/** The detected source type */
	sourceType: ImportSourceType;
	/** Suggested name for the instance */
	suggestedName?: string;
}

/** Progress event for import operations */
export interface ImportProgress {
	stage: string;
	progress: number;
	currentItem?: string;
}
