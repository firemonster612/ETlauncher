import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SvelteMap } from 'svelte/reactivity';
import type { TrackedTaskInfo } from '$lib/types';

/** Create the task manager store */
function createTaskManagerStore() {
	const taskMap = new SvelteMap<string, TrackedTaskInfo>();

	// Event listeners
	let unlistenUpdate: UnlistenFn | null = null;
	let unlistenRemoved: UnlistenFn | null = null;
	let initialized = false;

	// Track auto-remove timeouts so we can clear them on cleanup
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- internal timeout tracking, not rendered
	const autoRemoveTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	/** Schedule auto-removal of a task based on its status */
	function scheduleAutoRemove(task: TrackedTaskInfo) {
		// Clear any existing timeout for this task
		const existing = autoRemoveTimeouts.get(task.id);
		if (existing) {
			clearTimeout(existing);
			autoRemoveTimeouts.delete(task.id);
		}

		let delay: number | null = null;
		if (task.status === 'completed' || task.status === 'cancelled') {
			delay = 5000;
		} else if (task.status === 'failed') {
			delay = 10000;
		}

		if (delay !== null) {
			const timeout = setTimeout(() => {
				taskMap.delete(task.id);
				autoRemoveTimeouts.delete(task.id);
			}, delay);
			autoRemoveTimeouts.set(task.id, timeout);
		}
	}

	return {
		/** All tasks sorted by createdAt descending (newest first) */
		get tasks(): TrackedTaskInfo[] {
			return Array.from(taskMap.values()).sort((a, b) => b.createdAt - a.createdAt);
		},

		/** Tasks with status 'pending' or 'running' */
		get activeTasks(): TrackedTaskInfo[] {
			return this.tasks.filter((t) => t.status === 'pending' || t.status === 'running');
		},

		/** Count of active tasks */
		get activeCount(): number {
			return this.activeTasks.length;
		},

		/** Whether there are any active tasks */
		get hasActiveTasks(): boolean {
			return this.activeCount > 0;
		},

		/** Active tasks that are download-type (gameDownload, contentInstall, modpackInstall) */
		get activeDownloads(): TrackedTaskInfo[] {
			return this.activeTasks.filter(
				(t) =>
					t.taskType === 'gameDownload' ||
					t.taskType === 'contentInstall' ||
					t.taskType === 'modpackInstall'
			);
		},

		/** Initialize the store - call once at app level */
		async init() {
			if (initialized) return;
			initialized = true;

			// Get initial task list
			try {
				const tasks = await invoke<TrackedTaskInfo[]>('list_tasks');
				for (const task of tasks) {
					taskMap.set(task.id, task);
					scheduleAutoRemove(task);
				}
			} catch (e) {
				console.error('[taskManager] Failed to get initial tasks:', e);
			}

			// Listen for task updates (upsert)
			unlistenUpdate = await listen<TrackedTaskInfo>('task_update', (event) => {
				const task = event.payload;
				taskMap.set(task.id, task);
				scheduleAutoRemove(task);
			});

			// Listen for task removals
			unlistenRemoved = await listen<string>('task_removed', (event) => {
				const taskId = event.payload;
				taskMap.delete(taskId);
				const existing = autoRemoveTimeouts.get(taskId);
				if (existing) {
					clearTimeout(existing);
					autoRemoveTimeouts.delete(taskId);
				}
			});
		},

		/** Cancel a task */
		async cancelTask(taskId: string): Promise<boolean> {
			try {
				return await invoke<boolean>('cancel_task', { taskId });
			} catch (e) {
				console.error('[taskManager] Failed to cancel task:', e);
				return false;
			}
		},

		/** Dismiss a task (remove from backend and local map immediately) */
		async dismissTask(taskId: string) {
			// Remove from local map immediately for instant UI feedback
			taskMap.delete(taskId);
			const existing = autoRemoveTimeouts.get(taskId);
			if (existing) {
				clearTimeout(existing);
				autoRemoveTimeouts.delete(taskId);
			}

			try {
				await invoke('dismiss_task', { taskId });
			} catch (e) {
				console.error('[taskManager] Failed to dismiss task:', e);
			}
		},

		/** Cleanup event listeners */
		cleanup() {
			unlistenUpdate?.();
			unlistenRemoved?.();

			// Clear all auto-remove timeouts
			for (const timeout of autoRemoveTimeouts.values()) {
				clearTimeout(timeout);
			}
			autoRemoveTimeouts.clear();

			initialized = false;
		},
	};
}

export const taskManagerStore = createTaskManagerStore();
