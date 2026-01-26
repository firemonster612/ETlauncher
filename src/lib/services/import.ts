import { invoke } from '@tauri-apps/api/core';
import type { Instance, ImportAnalysis, ImportSourceType } from '$lib/types';

/** Analyze an import source to determine its type and extract metadata */
export async function analyzeImportSource(path: string): Promise<ImportAnalysis> {
	return invoke<ImportAnalysis>('analyze_import_source', { path });
}

/** Import from a folder (vanilla .minecraft or MultiMC/Prism) */
export async function importFromFolder(
	sourcePath: string,
	instanceName: string,
	sourceType: ImportSourceType
): Promise<Instance> {
	return invoke<Instance>('import_from_folder', { sourcePath, instanceName, sourceType });
}

/** Import a CurseForge modpack from a .zip file */
export async function importCurseForgeZip(
	filePath: string,
	instanceName?: string
): Promise<Instance> {
	return invoke<Instance>('import_curseforge_zip', { filePath, instanceName });
}

/** Export an instance to CurseForge .zip format */
export async function exportCurseForgeModpack(
	instanceId: string,
	outputPath: string
): Promise<string> {
	return invoke<string>('export_curseforge_modpack', { instanceId, outputPath });
}
