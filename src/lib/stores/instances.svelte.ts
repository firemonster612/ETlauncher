import type { Instance, CreateInstanceRequest, UpdateInstanceRequest } from "$lib/types";
import type { LoaderInstallProgress } from "$lib/types/loader";
import * as instanceService from "$lib/services/instance";
import * as loaderService from "$lib/services/loader";

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

    /** Load all instances from backend */
    async load() {
      isLoading = true;
      error = null;

      try {
        instances = await instanceService.getInstances();
      } catch (e: any) {
        error = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to load instances:", e);
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

        // If a non-vanilla loader is specified, install it
        if (request.loaderType && request.loaderType !== 'vanilla' && request.loaderVersion) {
          await this.installLoader(instance.id, request.loaderType, request.loaderVersion);
        }

        return instance;
      } catch (e: any) {
        error = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to create instance:", e);
        return null;
      }
    },

    /** Install a mod loader to an instance */
    async installLoader(
      instanceId: string,
      loaderType: string,
      loaderVersion: string
    ): Promise<boolean> {
      isInstallingLoader = true;
      loaderInstallProgress = null;
      loaderInstallError = null;

      try {
        await loaderService.installLoader(
          instanceId,
          loaderType as any,
          loaderVersion,
          (progress) => {
            loaderInstallProgress = progress;
          }
        );
        return true;
      } catch (e: any) {
        // Tauri errors are objects with message property
        loaderInstallError = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to install loader:", e);
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
      } catch (e: any) {
        error = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to update instance:", e);
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
      } catch (e: any) {
        error = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to delete instance:", e);
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
      } catch (e: any) {
        error = e?.message || (typeof e === 'string' ? e : JSON.stringify(e));
        console.error("Failed to duplicate instance:", e);
        return null;
      }
    },

    /** Clear error */
    clearError() {
      error = null;
    },
  };
}

/** Global instances store instance */
export const instancesStore = createInstancesStore();
