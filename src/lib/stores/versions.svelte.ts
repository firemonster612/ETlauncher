import type { VersionEntry, VersionManifest } from "$lib/types";
import * as minecraftService from "$lib/services/minecraft";

/** Create the versions store */
function createVersionsStore() {
  let versions = $state<VersionEntry[]>([]);
  let manifest = $state<VersionManifest | null>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // Filter settings
  let showSnapshots = $state(false);
  let showOldVersions = $state(false);

  return {
    // Getters
    get versions() {
      return versions;
    },
    get manifest() {
      return manifest;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },
    get showSnapshots() {
      return showSnapshots;
    },
    get showOldVersions() {
      return showOldVersions;
    },
    get latestRelease() {
      return manifest?.latest.release ?? null;
    },
    get latestSnapshot() {
      return manifest?.latest.snapshot ?? null;
    },

    /** Load versions from backend */
    async load(forceRefresh: boolean = false) {
      isLoading = true;
      error = null;

      try {
        // Fetch manifest for latest version info
        manifest = await minecraftService.fetchVersionManifest(forceRefresh);

        // Fetch filtered versions
        versions = await minecraftService.getVersions(showSnapshots, showOldVersions);
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        console.error("Failed to load versions:", e);
      } finally {
        isLoading = false;
      }
    },

    /** Set filter options and reload */
    async setFilters(snapshots: boolean, oldVersions: boolean) {
      showSnapshots = snapshots;
      showOldVersions = oldVersions;

      // Reload with new filters
      if (manifest) {
        try {
          versions = await minecraftService.getVersions(showSnapshots, showOldVersions);
        } catch (e) {
          error = e instanceof Error ? e.message : String(e);
        }
      }
    },

    /** Clear error */
    clearError() {
      error = null;
    },
  };
}

/** Global versions store instance */
export const versionsStore = createVersionsStore();
