import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

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

		/**
		 * Check for updates
		 * @param silent If true, don't show errors to the user (for automatic checks)
		 */
		async checkForUpdates(silent = false) {
			if (isChecking) return;

			isChecking = true;
			error = null;

			try {
				const update = await check();

				if (update) {
					updateAvailable = true;
					latestVersion = update.version;
					currentUpdate = update;
				} else {
					updateAvailable = false;
					latestVersion = null;
					currentUpdate = null;
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

				await currentUpdate.downloadAndInstall((event) => {
					switch (event.event) {
						case 'Started':
							contentLength = event.data.contentLength ?? 0;
							break;
						case 'Progress':
							downloaded += event.data.chunkLength;
							if (contentLength > 0) {
								downloadProgress = (downloaded / contentLength) * 100;
							}
							break;
						case 'Finished':
							downloadProgress = 100;
							break;
					}
				});

				// Relaunch the app after successful install
				await relaunch();
			} catch (e) {
				console.error('Failed to download and install update:', e);
				error = e instanceof Error ? e.message : 'Failed to install update';
			} finally {
				isDownloading = false;
			}
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
		},
	};
}

/** Global updater store instance */
export const updaterStore = createUpdaterStore();
