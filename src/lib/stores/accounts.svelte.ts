import * as accountService from '$lib/services/account';
import * as authService from '$lib/services/auth';
import type { DeviceCodeResponse, MinecraftAccount } from '$lib/types';

/** Extract error message from various error types */
function getErrorMessage(e: unknown): string {
	if (e instanceof Error) return e.message;
	if (typeof e === 'string') return e;
	if (e && typeof e === 'object') {
		// Tauri errors often have a message property
		if ('message' in e && typeof e.message === 'string') return e.message;
		// Some errors have an error property
		if ('error' in e && typeof e.error === 'string') return e.error;
		// Try JSON stringify as last resort
		try {
			return JSON.stringify(e);
		} catch {
			return 'An unknown error occurred';
		}
	}
	return 'An unknown error occurred';
}

/** Create the accounts store */
function createAccountsStore() {
	let accounts = $state<MinecraftAccount[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Auth state
	let isAuthenticating = $state(false);
	let deviceCode = $state<DeviceCodeResponse | null>(null);
	let authError = $state<string | null>(null);
	let pollInterval = $state<ReturnType<typeof setInterval> | null>(null);

	/** Enrich offline accounts with skin data URLs from local storage */
	async function enrichOfflineSkins(accountList: MinecraftAccount[]) {
		for (const account of accountList) {
			if (account.accountType === 'offline' && account.offlineSkinHash && !account.skinUrl) {
				try {
					const dataUrl = await authService.getOfflineSkinData(account.id);
					if (dataUrl) account.skinUrl = dataUrl;
				} catch {
					// ignore - skin just won't show
				}
			}
		}
	}

	return {
		// Getters
		get accounts() {
			return accounts;
		},
		get activeAccount() {
			return accounts.find((a) => a.isActive) ?? null;
		},
		get isLoading() {
			return isLoading;
		},
		get error() {
			return error;
		},
		get isAuthenticating() {
			return isAuthenticating;
		},
		get deviceCode() {
			return deviceCode;
		},
		get authError() {
			return authError;
		},

		/** Load accounts from backend, enriching offline accounts with skin data URLs */
		async load() {
			isLoading = true;
			error = null;

			try {
				const loaded = await accountService.getAccounts();
				await enrichOfflineSkins(loaded);
				accounts = loaded;
			} catch (e) {
				error = getErrorMessage(e);
				console.error('Failed to load accounts:', e);
			} finally {
				isLoading = false;
			}
		},

		/** Start device code authentication */
		async startAuth() {
			isAuthenticating = true;
			authError = null;
			deviceCode = null;

			try {
				deviceCode = await authService.startDeviceAuth();

				// Start polling
				pollInterval = setInterval(
					async () => {
						if (!deviceCode) return;

						try {
							const status = await authService.pollDeviceAuth(deviceCode.deviceCode);

							if (status.status === 'success') {
								// Auth succeeded
								this.stopAuth();
								const loaded = await accountService.getAccounts();
								await enrichOfflineSkins(loaded);
								accounts = loaded;
							} else if (status.status === 'expired') {
								authError = 'Authentication expired. Please try again.';
								this.stopAuth();
							} else if (status.status === 'error') {
								authError = status.message;
								this.stopAuth();
							}
							// status === "pending" - keep polling
						} catch (e) {
							authError = getErrorMessage(e);
							this.stopAuth();
						}
					},
					(deviceCode.interval || 5) * 1000
				);
			} catch (e) {
				authError = getErrorMessage(e);
				isAuthenticating = false;
			}
		},

		/** Stop authentication polling */
		stopAuth() {
			if (pollInterval) {
				clearInterval(pollInterval);
				pollInterval = null;
			}
			isAuthenticating = false;
			deviceCode = null;
		},

		/** Set active account */
		async setActive(accountId: string) {
			try {
				const updated = await accountService.setActiveAccount(accountId);
				await enrichOfflineSkins(updated);
				accounts = updated;
			} catch (e) {
				error = getErrorMessage(e);
				throw e;
			}
		},

		/** Create an offline account */
		async createOfflineAccount(username: string) {
			try {
				await authService.createOfflineAccount(username);
				const loaded = await accountService.getAccounts();
				await enrichOfflineSkins(loaded);
				accounts = loaded;
			} catch (e) {
				error = getErrorMessage(e);
				throw e;
			}
		},

		/** Delete account (logout) */
		async deleteAccount(accountId: string) {
			try {
				const updated = await accountService.deleteAccount(accountId);
				await enrichOfflineSkins(updated);
				accounts = updated;
			} catch (e) {
				error = getErrorMessage(e);
				throw e;
			}
		},

		/** Clear error */
		clearError() {
			error = null;
			authError = null;
		},
	};
}

/** Global accounts store instance */
export const accountsStore = createAccountsStore();
