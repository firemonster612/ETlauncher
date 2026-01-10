import type { LaunchStatus, GameLogLine } from "$lib/types";
import * as launchService from "$lib/services/launch";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SvelteMap } from "svelte/reactivity";
import { getErrorMessage } from "$lib/utils/error";

interface LaunchState {
  instanceId: string;
  status: LaunchStatus;
}

// Unique ID counter for log entries
let logIdCounter = 0;

/** Create the launch store */
function createLaunchStore() {
  let launchStates: Map<string, LaunchState> = new SvelteMap();
  let gameLogs = $state<(GameLogLine & { id: number })[]>([]);
  let error = $state<string | null>(null);

  // Event listeners
  let unlistenStatus: UnlistenFn | null = null;
  let unlistenLog: UnlistenFn | null = null;
  let initialized = false;

  // Log batching to prevent UI freeze from rapid log updates
  let pendingLogs: (GameLogLine & { id: number })[] = [];
  let logFlushTimeout: ReturnType<typeof setTimeout> | null = null;

  function flushLogs() {
    if (pendingLogs.length === 0) return;

    // Batch update logs
    const newLogs = [...gameLogs, ...pendingLogs];
    // Keep only last 1000 logs
    gameLogs = newLogs.length > 1000 ? newLogs.slice(-1000) : newLogs;
    pendingLogs = [];
    logFlushTimeout = null;
  }

  function queueLog(log: GameLogLine) {
    // Add unique ID to each log
    pendingLogs.push({ ...log, id: logIdCounter++ });
    // Flush logs every 100ms to batch updates
    if (!logFlushTimeout) {
      logFlushTimeout = setTimeout(flushLogs, 100);
    }
  }

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
    getLogsForInstance(instanceId: string) {
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
      launchStates = new SvelteMap(launchStates);

      try {
        const pid = await launchService.launchInstance(instanceId, accountId);
        return pid;
      } catch (e) {
        console.error("Launch error:", e);
        error = getErrorMessage(e);
        launchStates.delete(instanceId);
        launchStates = new SvelteMap(launchStates);
        return null;
      }
    },

    /** Initialize event listeners */
    async init() {
      // Guard against multiple init calls
      if (initialized) {
        console.log("[launchStore] Already initialized, skipping");
        return;
      }
      initialized = true;

      // Listen for launch status events
      unlistenStatus = await listen<{ instance_id: string; status: LaunchStatus }>(
        "launch_status",
        (event) => {
          const { instance_id, status } = event.payload;

          if (status.status === "stopped" || status.status === "crashed") {
            // Remove from active states after a delay
            setTimeout(() => {
              launchStates.delete(instance_id);
              launchStates = new SvelteMap(launchStates);
            }, 5000);
          }

          launchStates.set(instance_id, {
            instanceId: instance_id,
            status,
          });
          launchStates = new SvelteMap(launchStates);
        }
      );

      // Listen for game log events with batching to prevent UI freeze
      unlistenLog = await listen<GameLogLine>("game_log", (event) => {
        queueLog(event.payload);
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
        launchStates = new SvelteMap(launchStates);
      } catch (e) {
        console.error("Failed to get running instances:", e);
      }
    },

    /** Cleanup event listeners */
    cleanup() {
      unlistenStatus?.();
      unlistenLog?.();
      unlistenStatus = null;
      unlistenLog = null;
      initialized = false;
      // Flush any pending logs before cleanup
      if (logFlushTimeout) {
        clearTimeout(logFlushTimeout);
        logFlushTimeout = null;
      }
      flushLogs();
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

    /** Kill a running instance */
    async kill(instanceId: string): Promise<boolean> {
      try {
        await launchService.killInstance(instanceId);
        // The status will be updated via the event listener
        return true;
      } catch (e) {
        console.error("Kill error:", e);
        error = getErrorMessage(e);
        return false;
      }
    },
  };
}

/** Global launch store instance */
export const launchStore = createLaunchStore();
