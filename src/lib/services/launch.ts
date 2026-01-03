import { invoke } from "@tauri-apps/api/core";

/** Launch a Minecraft instance */
export async function launchInstance(instanceId: string, accountId: string): Promise<number> {
  return invoke<number>("launch_instance", { instanceId, accountId });
}

/** Check if an instance is running */
export async function isInstanceRunning(instanceId: string): Promise<boolean> {
  return invoke<boolean>("is_instance_running", { instanceId });
}

/** Get all running instances */
export async function getRunningInstances(): Promise<string[]> {
  return invoke<string[]>("get_running_instances");
}
