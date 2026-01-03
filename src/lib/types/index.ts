// Re-export all types
export * from "./account";
export * from "./instance";
export * from "./minecraft";
export * from "./settings";

/** Error returned from Tauri commands */
export interface CommandError {
  code: string;
  message: string;
}
