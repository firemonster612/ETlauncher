import type {
	Instance,
	CreateInstanceRequest,
	UpdateInstanceRequest,
	LoaderType,
	InstanceSetupStatus,
	DownloadProgress,
} from '$lib/types';
import type { LoaderInstallProgress } from '$lib/types/loader';
import * as instanceService from '$lib/services/instance';
import * as loaderService from '$lib/services/loader';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SvelteMap } from 'svelte/reactivity';

/** Create the instances store */
function createInstancesStore() {
	let instances = $state<Instance[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let selectedInstanceId = $state<string | null>(null);

	// Loader installation state
	let isInstallingLoader = $state(false);
	let loaderInstallProgress = $state<LoaderInstallProgress | null>(null);
	let loaderInstallError = $state<string | null>(null);

	// Instance setup state
	// eslint-disable-next-line svelte/no-unnecessary-state-wrap
	let setupStatuses: Map<string, InstanceSetupStatus> = $state(new SvelteMap());
	let setupError = $state<string | null>(null);

	// Event listeners
	let unlistenSetupStatus: UnlistenFn | null = null;
	let unlistenDownloadProgress: UnlistenFn | null = null;
	let initialized = false;

	return {
		// Getters
		get instances() {
			return instances;
		},
		get isLoading() {
			return isLoading;
		},
		get error() {
			return error;
		},
		get selectedInstanceId() {
			return selectedInstanceId;
		},
		get selectedInstance() {
			return instances.find((i) => i.id === selectedInstanceId) ?? null;
		},
		get isInstallingLoader() {
			return isInstallingLoader;
		},
		get loaderInstallProgress() {
			return loaderInstallProgress;
		},
		get loaderInstallError() {
			return loaderInstallError;
		},
		get setupStatuses() {
			return setupStatuses;
		},
		get setupError() {
			return setupError;
		},

		/** Load all instances from backend */
		async load() {
			isLoading = true;
			error = null;

			try {
				instances = await instanceService.getInstances();
			} catch (e: unknown) {
				error = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to load instances:', e);
			} finally {
				isLoading = false;
			}
		},

		/** Select an instance */
		select(instanceId: string | null) {
			selectedInstanceId = instanceId;
		},

		/** Create a new instance */
		async create(request: CreateInstanceRequest): Promise<Instance | null> {
			error = null;

			try {
				const instance = await instanceService.createInstance(request);
				instances = [instance, ...instances];

				// If a non-vanilla loader is specified, install it first
				if (request.loaderType && request.loaderType !== 'vanilla' && request.loaderVersion) {
					const loaderSuccess = await this.installLoader(
						instance.id,
						request.loaderType,
						request.loaderVersion
					);
					if (!loaderSuccess) {
						// Loader failed, but instance was created - don't setup yet
						return instance;
					}
				}

				// Setup instance (download game files) - runs in background
				// Don't await - let it run asynchronously
				this.setupInstance(instance.id).catch((e) => {
					console.error('Instance setup failed:', e);
				});

				return instance;
			} catch (e: unknown) {
				error = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to create instance:', e);
				return null;
			}
		},

		/** Install a mod loader to an instance */
		async installLoader(
			instanceId: string,
			loaderType: LoaderType,
			loaderVersion: string
		): Promise<boolean> {
			isInstallingLoader = true;
			loaderInstallProgress = null;
			loaderInstallError = null;

			try {
				await loaderService.installLoader(instanceId, loaderType, loaderVersion, (progress) => {
					loaderInstallProgress = progress;
				});
				return true;
			} catch (e: unknown) {
				// Tauri errors are objects with message property
				loaderInstallError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to install loader:', e);
				return false;
			} finally {
				isInstallingLoader = false;
			}
		},

		/** Clear loader installation error */
		clearLoaderError() {
			loaderInstallError = null;
		},

		/** Update an existing instance */
		async update(instanceId: string, updates: UpdateInstanceRequest): Promise<Instance | null> {
			error = null;

			try {
				const updated = await instanceService.updateInstance(instanceId, updates);
				instances = instances.map((i) => (i.id === instanceId ? updated : i));
				return updated;
			} catch (e: unknown) {
				error = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to update instance:', e);
				return null;
			}
		},

		/** Delete an instance */
		async delete(instanceId: string, deleteFiles: boolean): Promise<boolean> {
			error = null;

			try {
				await instanceService.deleteInstance(instanceId, deleteFiles);
				instances = instances.filter((i) => i.id !== instanceId);
				if (selectedInstanceId === instanceId) {
					selectedInstanceId = null;
				}
				return true;
			} catch (e: unknown) {
				error = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to delete instance:', e);
				return false;
			}
		},

		/** Duplicate an instance */
		async duplicate(instanceId: string, newName: string): Promise<Instance | null> {
			error = null;

			try {
				const duplicate = await instanceService.duplicateInstance(instanceId, newName);
				instances = [duplicate, ...instances];
				return duplicate;
			} catch (e: unknown) {
				error = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to duplicate instance:', e);
				return null;
			}
		},

		/** Clear error */
		clearError() {
			error = null;
		},

		/** Clear setup error */
		clearSetupError() {
			setupError = null;
		},

		/** Get setup status for a specific instance */
		getSetupStatus(instanceId: string): InstanceSetupStatus | null {
			return setupStatuses.get(instanceId) ?? null;
		},

		/** Check if an instance is currently being set up */
		isSettingUp(instanceId: string): boolean {
			const status = setupStatuses.get(instanceId);
			if (!status) return false;
			return (
				status.status === 'pending' ||
				status.status === 'preparing' ||
				status.status === 'downloadingGameFiles' ||
				status.status === 'installingLoader'
			);
		},

		/** Setup an instance by downloading game files */
		async setupInstance(instanceId: string): Promise<boolean> {
			setupError = null;

			// Set initial pending status
			setupStatuses.set(instanceId, { status: 'pending' });
			setupStatuses = new SvelteMap(setupStatuses);

			try {
				await instanceService.setupInstance(instanceId);
				return true;
			} catch (e: unknown) {
				setupError = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				setupStatuses.set(instanceId, {
					status: 'failed',
					message: setupError,
				});
				setupStatuses = new SvelteMap(setupStatuses);
				console.error('Failed to setup instance:', e);
				return false;
			}
		},

		/** Initialize event listeners */
		async init() {
			if (initialized) {
				console.log('[instancesStore] Already initialized, skipping');
				return;
			}
			initialized = true;

			// Listen for instance setup status events
			unlistenSetupStatus = await listen<{ instance_id: string; status: InstanceSetupStatus }>(
				'instance_setup_status',
				(event) => {
					const { instance_id, status } = event.payload;

					setupStatuses.set(instance_id, status);
					setupStatuses = new SvelteMap(setupStatuses);

					// Clean up completed/failed statuses after a delay
					if (status.status === 'complete' || status.status === 'failed') {
						setTimeout(() => {
							setupStatuses.delete(instance_id);
							setupStatuses = new SvelteMap(setupStatuses);
						}, 5000);
					}
				}
			);

			// Listen for download progress events to update setup status
			unlistenDownloadProgress = await listen<DownloadProgress>('download_progress', (event) => {
				// Find which instance this download is for by checking active setup statuses
				// We update all instances in "downloadingGameFiles" state since the backend
				// only sends one instance's downloads at a time
				for (const [instanceId, status] of setupStatuses.entries()) {
					if (
						status.status === 'pending' ||
						status.status === 'preparing' ||
						status.status === 'downloadingGameFiles'
					) {
						setupStatuses.set(instanceId, {
							status: 'downloadingGameFiles',
							progress: event.payload,
						});
						setupStatuses = new SvelteMap(setupStatuses);
						break;
					}
				}
			});
		},

		/** Cleanup event listeners */
		cleanup() {
			unlistenSetupStatus?.();
			unlistenDownloadProgress?.();
			unlistenSetupStatus = null;
			unlistenDownloadProgress = null;
			initialized = false;
		},
	};
}

/** Global instances store instance */
export const instancesStore = createInstancesStore();
