<script lang="ts">
	import { onMount } from 'svelte';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { Trash2, Copy } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import * as Select from '$lib/ui/select';
	import { Checkbox } from '$lib/ui/checkbox';

	let selectedInstanceId = $state<string>('');
	let logContainer = $state<HTMLDivElement | null>(null);
	let autoScroll = $state(true);

	onMount(() => {
		// launchStore is initialized at app layout level
		instancesStore.load();
	});

	const filteredLogs = $derived(
		selectedInstanceId && selectedInstanceId !== ''
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
			case 'error':
				return 'text-destructive';
			case 'warn':
				return 'text-amber-500';
			case 'debug':
				return 'text-muted-foreground';
			default:
				return 'text-foreground';
		}
	}

	function formatTimestamp(timestamp: number): string {
		return new Date(timestamp).toLocaleTimeString();
	}

	function copyLogs() {
		const text = filteredLogs
			.map((log) => `[${formatTimestamp(log.timestamp)}] ${log.line}`)
			.join('\n');
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
				<Select.Trigger class="border-border bg-card w-[180px] border-2">
					{#if selectedInstanceId}
						{instancesStore.instances.find((i) => i.id === selectedInstanceId)?.name ?? 'Unknown'}
					{:else}
						All Instances
					{/if}
				</Select.Trigger>
				<Select.Content class="border-border bg-card border-2">
					<Select.Item value="" label="All Instances">All Instances</Select.Item>
					{#each instancesStore.instances as instance (instance.id)}
						<Select.Item value={instance.id} label={instance.name}>{instance.name}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
			<div class="flex cursor-pointer items-center gap-2 text-sm">
				<Checkbox id="auto-scroll-checkbox" bind:checked={autoScroll} />
				<label for="auto-scroll-checkbox">Auto-scroll</label>
			</div>
		</div>
	</div>

	<div class="border-border bg-card flex min-h-0 flex-1 flex-col border-2">
		<div class="border-border flex shrink-0 items-center justify-between border-b-2 px-4 py-2">
			<span class="text-sm tracking-wider uppercase">Output</span>
			<div class="flex gap-2">
				<Button variant="ghost" size="sm" onclick={clearLogs}>
					<Trash2 class="mr-1 h-4 w-4" />
					Clear
				</Button>
				<Button variant="ghost" size="sm" onclick={copyLogs}>
					<Copy class="mr-1 h-4 w-4" />
					Copy
				</Button>
			</div>
		</div>

		<div class="flex-1 overflow-auto p-4 font-mono text-sm" bind:this={logContainer}>
			{#if filteredLogs.length === 0}
				<div class="text-muted-foreground py-8 text-center">
					{#if [...launchStore.launchStates.values()].some((s) => s.status.status === 'running')}
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
		<div class="border-border bg-card border-2 p-4">
			<h2 class="text-muted-foreground mb-2 text-sm tracking-wider uppercase">Running Instances</h2>
			<div class="space-y-2">
				{#each [...launchStore.launchStates.entries()] as [instanceId, state] (instanceId)}
					{@const instance = instancesStore.instances.find((i) => i.id === instanceId)}
					<div class="flex items-center justify-between text-sm">
						<span>{instance?.name ?? instanceId}</span>
						<span class="text-muted-foreground capitalize">
							{#if state.status.status === 'checkingAccount'}
								Checking account...
							{:else if state.status.status === 'refreshingToken'}
								Refreshing token...
							{:else if state.status.status === 'loadingVersion'}
								Loading version...
							{:else if state.status.status === 'verifyingFiles'}
								Verifying files ({state.status.checked}/{state.status.total})
							{:else if state.status.status === 'downloading'}
								Downloading ({state.status.progress.completedFiles}/{state.status.progress
									.totalFiles})
							{:else if state.status.status === 'checkingJava'}
								Checking Java {state.status.version}...
							{:else if state.status.status === 'downloadingJava'}
								Downloading Java {state.status.version} ({state.status.progress}%)
							{:else if state.status.status === 'buildingClasspath'}
								Building classpath...
							{:else if state.status.status === 'launching'}
								Launching...
							{:else if state.status.status === 'running'}
								Starting... (PID: {state.status.pid})
							{:else if state.status.status === 'windowReady'}
								Running (PID: {state.status.pid})
							{:else if state.status.status === 'stopped'}
								Stopped (exit: {state.status.exitCode})
							{:else if state.status.status === 'crashed'}
								Crashed: {state.status.message}
							{:else}
								{state.status.status}
							{/if}
						</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
