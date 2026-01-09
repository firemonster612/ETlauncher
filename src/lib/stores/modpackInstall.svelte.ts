import type { ModpackInstallProgress } from "$lib/types";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

/** Info sent when a modpack install starts */
interface ModpackInstallStarted {
  modpackName: string;
}

/** Create the modpack install store */
function createModpackInstallStore() {
  let isInstalling = $state(false);
  let modpackName = $state<string | null>(null);
  let progress = $state<ModpackInstallProgress | null>(null);
  let error = $state<string | null>(null);
  let isCancelling = $state(false);

  // Event listeners
  let unlistenStarted: UnlistenFn | null = null;
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;
  let unlistenCancelled: UnlistenFn | null = null;
  let initialized = false;

  return {
    // Getters
    get isInstalling() {
      return isInstalling;
    },
    get modpackName() {
      return modpackName;
    },
    get progress() {
      return progress;
    },
    get error() {
      return error;
    },
    get isCancelling() {
      return isCancelling;
    },

    /** Initialize the store - call once at app level */
    async init() {
      if (initialized) return;
      initialized = true;

      // Check if there's an ongoing install (in case of app restart)
      try {
        const status = await invoke<ModpackInstallStarted | null>(
          "get_modpack_install_status"
        );
        if (status) {
          isInstalling = true;
          modpackName = status.modpackName;
        }
      } catch (e) {
        console.error(
          "[modpackInstallStore] Failed to get initial status:",
          e
        );
      }

      // Listen for install started
      unlistenStarted = await listen<ModpackInstallStarted>(
        "modpack_install_started",
        (event) => {
          console.log("[modpackInstallStore] Install started:", event.payload);
          isInstalling = true;
          modpackName = event.payload.modpackName;
          progress = null;
          error = null;
          isCancelling = false;
        }
      );

      // Listen for progress updates
      unlistenProgress = await listen<ModpackInstallProgress>(
        "modpack_install_progress",
        (event) => {
          progress = event.payload;
        }
      );

      // Listen for completion
      unlistenComplete = await listen("modpack_install_complete", () => {
        console.log("[modpackInstallStore] Install complete");
        isInstalling = false;
        modpackName = null;
        progress = null;
        isCancelling = false;
      });

      // Listen for error
      unlistenError = await listen<string>("modpack_install_error", (event) => {
        console.error("[modpackInstallStore] Install error:", event.payload);
        error = event.payload;
        isInstalling = false;
        modpackName = null;
        progress = null;
        isCancelling = false;
      });

      // Listen for cancellation
      unlistenCancelled = await listen("modpack_install_cancelled", () => {
        console.log("[modpackInstallStore] Install cancelled");
        isInstalling = false;
        modpackName = null;
        progress = null;
        isCancelling = false;
      });
    },

    /** Cancel the current installation */
    async cancel() {
      if (!isInstalling || isCancelling) return;
      isCancelling = true;
      try {
        await invoke("cancel_modpack_install");
      } catch (e) {
        console.error("[modpackInstallStore] Cancel failed:", e);
        isCancelling = false;
      }
    },

    /** Clear the error state */
    clearError() {
      error = null;
    },

    /** Cleanup event listeners */
    cleanup() {
      unlistenStarted?.();
      unlistenProgress?.();
      unlistenComplete?.();
      unlistenError?.();
      unlistenCancelled?.();
      initialized = false;
    },
  };
}

export const modpackInstallStore = createModpackInstallStore();
