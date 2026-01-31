import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { invoke } from '@tauri-apps/api/core';

/** Check if running from an AppImage on Linux (or non-Linux platforms) */
async function checkCanAutoUpdate(): Promise<boolean> {
	try {
		// Check if running from AppImage (Linux) - uses APPIMAGE env var
		const isAppImage = await invoke<boolean>('is_appimage').catch(() => false);
		if (isAppImage) {
			return true;
		}

		// Get executable path to determine install type
		const exePath = await invoke<string>('get_exe_path').catch(() => '');

		// Check for common Linux system paths (DEB/RPM installs)
		// These indicate a system package installation that can't self-update
		if (exePath.startsWith('/usr/') || exePath.startsWith('/opt/') || exePath.includes('/bin/')) {
			return false;
		}

		// For Windows (.exe) and macOS (.app), auto-update is supported
		// Also allow if we can't determine (e.g., dev builds)
		return true;
	} catch {
		// If we can't determine, assume we can auto-update (better UX)
		return true;
	}
}

/** Create the updater store */
function createUpdaterStore() {
	// State
	let isChecking = $state(false);
	let isDownloading = $state(false);
	let updateAvailable = $state(false);
	let latestVersion = $state<string | null>(null);
	let downloadProgress = $state(0);
	let currentUpdate = $state<Update | null>(null);
	let error = $state<string | null>(null);
	let canAutoUpdate = $state(true); // Assume true until checked
	let showNotification = $state(false); // Whether to show the floating notification

	return {
		// Getters
		get isChecking() {
			return isChecking;
		},
		get isDownloading() {
			return isDownloading;
		},
		get updateAvailable() {
			return updateAvailable;
		},
		get latestVersion() {
			return latestVersion;
		},
		get downloadProgress() {
			return downloadProgress;
		},
		get error() {
			return error;
		},
		get canAutoUpdate() {
			return canAutoUpdate;
		},
		get showNotification() {
			return showNotification;
		},

		/**
		 * Check for updates
		 * @param options.silent If true, don't show errors to the user (for automatic checks)
		 * @param options.showNotification If true, show the floating notification (default: true for automatic, false for manual)
		 */
		async checkForUpdates(options: { silent?: boolean; showNotification?: boolean } = {}) {
			if (isChecking) return;

			const { silent = false, showNotification: shouldShowNotification = true } = options;

			isChecking = true;
			error = null;

			try {
				// Check if we can auto-update (only matters on Linux)
				canAutoUpdate = await checkCanAutoUpdate();

				const update = await check();

				if (update) {
					updateAvailable = true;
					latestVersion = update.version;
					currentUpdate = update;
					showNotification = shouldShowNotification;
				} else {
					updateAvailable = false;
					latestVersion = null;
					currentUpdate = null;
					showNotification = false;
				}
			} catch (e) {
				console.error('Failed to check for updates:', e);
				if (!silent) {
					error = e instanceof Error ? e.message : 'Failed to check for updates';
				}
			} finally {
				isChecking = false;
			}
		},

		/**
		 * Download and install the update
		 */
		async downloadAndInstall() {
			if (!currentUpdate || isDownloading) return;

			isDownloading = true;
			downloadProgress = 0;
			error = null;

			try {
				let downloaded = 0;
				let contentLength = 0;

				console.log('[Updater] Starting download and install for version:', currentUpdate.version);

				await currentUpdate.downloadAndInstall((event) => {
					switch (event.event) {
						case 'Started':
							contentLength = event.data.contentLength ?? 0;
							console.log('[Updater] Download started, content length:', contentLength);
							break;
						case 'Progress':
							downloaded += event.data.chunkLength;
							if (contentLength > 0) {
								downloadProgress = (downloaded / contentLength) * 100;
							}
							break;
						case 'Finished':
							downloadProgress = 100;
							console.log('[Updater] Download finished, installing...');
							break;
					}
				});

				console.log('[Updater] Install complete, relaunching...');
				// Relaunch the app after successful install
				await relaunch();
			} catch (e) {
				console.error('[Updater] Failed to download and install update:', e);
				// Provide more specific error messages based on common issues
				let errorMessage = 'Failed to install update';
				if (e instanceof Error) {
					const msg = e.message.toLowerCase();
					if (msg.includes('signature') || msg.includes('invalid')) {
						errorMessage = 'Update signature verification failed. The update may be corrupted.';
					} else if (msg.includes('permission') || msg.includes('access denied')) {
						errorMessage = 'Permission denied. Try running as administrator.';
					} else if (msg.includes('move') || msg.includes('replace')) {
						errorMessage =
							'Failed to replace app files. Close any file managers viewing the app folder.';
					} else if (msg.includes('kill') || msg.includes('close')) {
						errorMessage = 'Failed to close the app for update. Please restart and try again.';
					} else if (msg.includes('404') || msg.includes('not found')) {
						errorMessage = 'Update file not found. Please try again later.';
					} else if (msg.includes('timeout')) {
						errorMessage = 'Download timed out. Check your internet connection.';
					} else {
						errorMessage = e.message;
					}
				}
				error = errorMessage;
			} finally {
				isDownloading = false;
			}
		},

		/**
		 * Dismiss the floating notification without clearing update state
		 */
		dismissNotification() {
			showNotification = false;
		},

		/**
		 * Clear the current update state
		 */
		clearUpdate() {
			updateAvailable = false;
			latestVersion = null;
			currentUpdate = null;
			error = null;
			downloadProgress = 0;
			showNotification = false;
		},
	};
}

/** Global updater store instance */
export const updaterStore = createUpdaterStore();
