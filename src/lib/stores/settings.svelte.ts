import type { AppSettings } from "$lib/types";
import * as settingsService from "$lib/services/settings";

/** Settings store state */
interface SettingsState {
  settings: AppSettings | null;
  isLoading: boolean;
  error: string | null;
}

/** Create the settings store */
function createSettingsStore() {
  let settings = $state<AppSettings | null>(null);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  return {
    // Getters
    get settings() {
      return settings;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },

    /** Load settings from backend */
    async load() {
      isLoading = true;
      error = null;

      try {
        settings = await settingsService.getSettings();
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        console.error("Failed to load settings:", e);
      } finally {
        isLoading = false;
      }
    },

    /** Update settings */
    async update(updates: Partial<AppSettings>) {
      isLoading = true;
      error = null;

      try {
        settings = await settingsService.updateSettings(updates);
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        console.error("Failed to update settings:", e);
        throw e;
      } finally {
        isLoading = false;
      }
    },

    /** Reset settings to defaults */
    async reset() {
      isLoading = true;
      error = null;

      try {
        settings = await settingsService.resetSettings();
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        console.error("Failed to reset settings:", e);
        throw e;
      } finally {
        isLoading = false;
      }
    },

    /** Clear error */
    clearError() {
      error = null;
    },
  };
}

/** Global settings store instance */
export const settingsStore = createSettingsStore();
