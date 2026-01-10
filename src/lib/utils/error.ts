/** Extract error message from various error types (including Tauri CommandError) */
export function getErrorMessage(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  if (e && typeof e === "object") {
    if ("message" in e && typeof e.message === "string") return e.message;
    if ("error" in e && typeof e.error === "string") return e.error;
    try {
      return JSON.stringify(e);
    } catch {
      return "An unknown error occurred";
    }
  }
  return "An unknown error occurred";
}
