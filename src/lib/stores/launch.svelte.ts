import type { LaunchStatus, GameLogLine } from "$lib/types";
import * as launchService from "$lib/services/launch";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Extract error message from various error types */
function getErrorMessage(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  if (e && typeof e === "object") {
    if ("message" in e && typeof e.message === "string") return e.message;
    if ("error" in e && typeof e.error === "string") return e.error;
    try {
      return JSON.stringify(e);
    } catch {
      return "An unknown error occurred";
    }
  }
  return "An unknown error occurred";
}

interface LaunchState {
  instanceId: string;
  status: LaunchStatus;
}

/** Create the launch store */
function createLaunchStore() {
  let launchStates = $state<Map<string, LaunchState>>(new Map());
  let gameLogs = $state<GameLogLine[]>([]);
  let error = $state<string | null>(null);

  // Event listeners
  let unlistenStatus: UnlistenFn | null = null;
  let unlistenLog: UnlistenFn | null = null;

  return {
    // Getters
    get launchStates() {
      return launchStates;
    },
    get gameLogs() {
      return gameLogs;
    },
    get error() {
      return error;
    },

    /** Get launch status for a specific instance */
    getStatus(instanceId: string): LaunchStatus | null {
      return launchStates.get(instanceId)?.status ?? null;
    },

    /** Check if an instance is running */
    isRunning(instanceId: string): boolean {
      const status = launchStates.get(instanceId)?.status;
      return status?.status === "running";
    },

    /** Get logs for a specific instance */
    getLogsForInstance(instanceId: string): GameLogLine[] {
      return gameLogs.filter((log) => log.instanceId === instanceId);
    },

    /** Launch an instance */
    async launch(instanceId: string, accountId: string): Promise<number | null> {
      error = null;

      // Set initial preparing state
      launchStates.set(instanceId, {
        instanceId,
        status: { status: "preparing", message: "Starting..." },
      });
      launchStates = new Map(launchStates);

      try {
        const pid = await launchService.launchInstance(instanceId, accountId);
        return pid;
      } catch (e) {
        console.error("Launch error:", e);
        error = getErrorMessage(e);
        launchStates.delete(instanceId);
        launchStates = new Map(launchStates);
        return null;
      }
    },

    /** Initialize event listeners */
    async init() {
      // Listen for launch status events
      unlistenStatus = await listen<{ instance_id: string; status: LaunchStatus }>(
        "launch_status",
        (event) => {
          const { instance_id, status } = event.payload;

          if (status.status === "stopped" || status.status === "crashed") {
            // Remove from active states after a delay
            setTimeout(() => {
              launchStates.delete(instance_id);
              launchStates = new Map(launchStates);
            }, 5000);
          }

          launchStates.set(instance_id, {
            instanceId: instance_id,
            status,
          });
          launchStates = new Map(launchStates);
        }
      );

      // Listen for game log events
      unlistenLog = await listen<GameLogLine>("game_log", (event) => {
        gameLogs = [...gameLogs, event.payload];

        // Keep only last 1000 logs
        if (gameLogs.length > 1000) {
          gameLogs = gameLogs.slice(-1000);
        }
      });

      // Get initially running instances
      try {
        const running = await launchService.getRunningInstances();
        for (const instanceId of running) {
          launchStates.set(instanceId, {
            instanceId,
            status: { status: "running", pid: 0 },
          });
        }
        launchStates = new Map(launchStates);
      } catch (e) {
        console.error("Failed to get running instances:", e);
      }
    },

    /** Cleanup event listeners */
    cleanup() {
      unlistenStatus?.();
      unlistenLog?.();
    },

    /** Clear logs for an instance */
    clearLogs(instanceId?: string) {
      if (instanceId) {
        gameLogs = gameLogs.filter((log) => log.instanceId !== instanceId);
      } else {
        gameLogs = [];
      }
    },

    /** Clear error */
    clearError() {
      error = null;
    },
  };
}

/** Global launch store instance */
export const launchStore = createLaunchStore();
