<script lang="ts">
  import { onMount } from "svelte";
  import { launchStore } from "$lib/stores/launch.svelte";
  import { instancesStore } from "$lib/stores/instances.svelte";
  import { Trash2, Copy } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import * as Select from "$lib/ui/select";
  import { Checkbox } from "$lib/ui/checkbox";

  let selectedInstanceId = $state<string>("");
  let logContainer = $state<HTMLDivElement | null>(null);
  let autoScroll = $state(true);

  onMount(() => {
    // launchStore is initialized at app layout level
    instancesStore.load();
  });

  const filteredLogs = $derived(
    selectedInstanceId && selectedInstanceId !== ""
      ? launchStore.getLogsForInstance(selectedInstanceId)
      : launchStore.gameLogs
  );

  // Auto-scroll to bottom when new logs arrive
  $effect(() => {
    if (filteredLogs.length > 0 && autoScroll && logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  function getLogColor(level: string): string {
    switch (level) {
      case "error":
        return "text-destructive";
      case "warn":
        return "text-amber-500";
      case "debug":
        return "text-muted-foreground";
      default:
        return "text-foreground";
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  function copyLogs() {
    const text = filteredLogs.map((log) => `[${formatTimestamp(log.timestamp)}] ${log.line}`).join("\n");
    navigator.clipboard.writeText(text);
  }

  function clearLogs() {
    launchStore.clearLogs(selectedInstanceId || undefined);
  }
</script>

<div class="flex h-full flex-col space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl">Console</h1>
    <div class="flex items-center gap-4">
      <Select.Root type="single" bind:value={selectedInstanceId}>
        <Select.Trigger class="w-[180px] border-2 border-border bg-card">
          {#if selectedInstanceId}
            {instancesStore.instances.find((i) => i.id === selectedInstanceId)?.name ?? "Unknown"}
          {:else}
            All Instances
          {/if}
        </Select.Trigger>
        <Select.Content class="border-2 border-border bg-card">
          <Select.Item value="" label="All Instances">All Instances</Select.Item>
          {#each instancesStore.instances as instance (instance.id)}
            <Select.Item value={instance.id} label={instance.name}>{instance.name}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
      <label class="flex items-center gap-2 text-sm cursor-pointer">
        <Checkbox bind:checked={autoScroll} />
        Auto-scroll
      </label>
    </div>
  </div>

  <div class="flex-1 border-2 border-border bg-card flex flex-col min-h-0">
    <div class="flex items-center justify-between border-b-2 border-border px-4 py-2 shrink-0">
      <span class="text-sm uppercase tracking-wider">Output</span>
      <div class="flex gap-2">
        <Button variant="ghost" size="sm" onclick={clearLogs}>
          <Trash2 class="h-4 w-4 mr-1" />
          Clear
        </Button>
        <Button variant="ghost" size="sm" onclick={copyLogs}>
          <Copy class="h-4 w-4 mr-1" />
          Copy
        </Button>
      </div>
    </div>

    <div
      class="flex-1 overflow-auto p-4 font-mono text-sm"
      bind:this={logContainer}
    >
      {#if filteredLogs.length === 0}
        <div class="text-muted-foreground text-center py-8">
          {#if [...launchStore.launchStates.values()].some((s) => s.status.status === "running")}
            Waiting for game output...
          {:else}
            No game running. Launch an instance to see logs here.
          {/if}
        </div>
      {:else}
        <div class="space-y-0.5">
          {#each filteredLogs as log (log.id)}
            <div class="flex gap-2 {getLogColor(log.level)}">
              <span class="text-muted-foreground shrink-0">[{formatTimestamp(log.timestamp)}]</span>
              <span class="break-all">{log.line}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Running instances status -->
  {#if launchStore.launchStates.size > 0}
    <div class="border-2 border-border bg-card p-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground mb-2">Running Instances</h3>
      <div class="space-y-2">
        {#each [...launchStore.launchStates.entries()] as [instanceId, state]}
          {@const instance = instancesStore.instances.find((i) => i.id === instanceId)}
          <div class="flex items-center justify-between text-sm">
            <span>{instance?.name ?? instanceId}</span>
            <span class="text-muted-foreground capitalize">
              {state.status.status}
              {#if state.status.status === "preparing" && "message" in state.status}
                - {state.status.message}
              {/if}
              {#if state.status.status === "running" && "pid" in state.status}
                (PID: {state.status.pid})
              {/if}
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
