import { invoke } from "@tauri-apps/api/core";
import type {
  Content,
  ContentPlatform,
  ContentSearchParams,
  ContentSearchResult,
  ContentType,
  ContentVersion,
  InstalledContent,
  LoaderType,
  ResolvedDependency,
  ScanResult,
} from "$lib/types";

/** Search for content (mods, shaders, resource packs) across platforms */
export async function searchContent(
  params: ContentSearchParams
): Promise<ContentSearchResult> {
  return invoke<ContentSearchResult>("search_content", { params });
}

/** Get content by ID */
export async function getContent(
  platform: ContentPlatform,
  id: string
): Promise<Content> {
  return invoke<Content>("get_content", { platform, id });
}

/** Get versions for content */
export async function getContentVersions(
  platform: ContentPlatform,
  id: string,
  mcVersion?: string,
  loader?: LoaderType
): Promise<ContentVersion[]> {
  return invoke<ContentVersion[]>("get_content_versions", {
    platform,
    id,
    mcVersion,
    loader,
  });
}

/** Get a specific version by ID */
export async function getContentVersion(
  platform: ContentPlatform,
  versionId: string,
  modId?: string
): Promise<ContentVersion> {
  return invoke<ContentVersion>("get_content_version", { platform, versionId, modId });
}

/** Install content (mod, shader, resource pack) to an instance */
export async function installContent(
  instanceId: string,
  platform: ContentPlatform,
  contentId: string,
  contentName: string,
  contentSlug: string,
  contentType: ContentType,
  version: ContentVersion,
  isDependency?: boolean
): Promise<InstalledContent> {
  return invoke<InstalledContent>("install_content", {
    instanceId,
    platform,
    contentId,
    contentName,
    contentSlug,
    contentType,
    version,
    isDependency,
  });
}

/** Resolve dependencies for a content version */
export async function resolveContentDependencies(
  instanceId: string,
  platform: ContentPlatform,
  version: ContentVersion,
  mcVersion: string,
  loader?: LoaderType
): Promise<ResolvedDependency[]> {
  return invoke<ResolvedDependency[]>("resolve_content_dependencies", {
    instanceId,
    platform,
    version,
    mcVersion,
    loader,
  });
}

/** Install content with its dependencies */
export async function installContentWithDependencies(
  instanceId: string,
  platform: ContentPlatform,
  content: Content,
  version: ContentVersion,
  mcVersion: string,
  loader?: LoaderType
): Promise<InstalledContent[]> {
  return invoke<InstalledContent[]>("install_content_with_dependencies", {
    instanceId,
    platform,
    content,
    version,
    mcVersion,
    loader,
  });
}

/** Scan an instance's content folder and identify installed items via Modrinth hash lookup */
export async function scanInstalledContent(
  instanceId: string,
  contentType: ContentType
): Promise<ScanResult> {
  return invoke<ScanResult>("scan_installed_content", { instanceId, contentType });
}

/** Uninstall content by filename (delete the file directly) */
export async function uninstallContentByFilename(
  instanceId: string,
  filename: string,
  contentType: ContentType
): Promise<void> {
  return invoke<void>("uninstall_content_by_filename", {
    instanceId,
    filename,
    contentType,
  });
}

/** Disable content by moving files to the disabled subfolder */
export async function disableContent(
  instanceId: string,
  filenames: string[],
  contentType: ContentType
): Promise<void> {
  return invoke<void>("disable_content", {
    instanceId,
    filenames,
    contentType,
  });
}

/** Enable content by moving files from disabled subfolder back */
export async function enableContent(
  instanceId: string,
  filenames: string[],
  contentType: ContentType
): Promise<void> {
  return invoke<void>("enable_content", {
    instanceId,
    filenames,
    contentType,
  });
}
