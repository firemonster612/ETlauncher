import type { Instance, CreateInstanceRequest, UpdateInstanceRequest } from "$lib/types";
import * as instanceService from "$lib/services/instance";

/** Create the instances store */
function createInstancesStore() {
  let instances = $state<Instance[]>([]);
  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let selectedInstanceId = $state<string | null>(null);

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

    /** Load all instances from backend */
    async load() {
      isLoading = true;
      error = null;

      try {
        instances = await instanceService.getInstances();
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
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
        return instance;
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        console.error("Failed to create instance:", e);
        return null;
      }
    },

    /** Update an existing instance */
    async update(instanceId: string, updates: UpdateInstanceRequest): Promise<Instance | null> {
      error = null;

      try {
        const updated = await instanceService.updateInstance(instanceId, updates);
        instances = instances.map((i) => (i.id === instanceId ? updated : i));
        return updated;
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
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
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
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
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
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
