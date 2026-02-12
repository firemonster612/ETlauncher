<script lang="ts">
	import {
		Activity,
		ArrowUpDown,
		ChevronUp,
		Download,
		Import,
		Loader2,
		Package,
		RefreshCw,
		Rocket,
		Search,
		Upload,
		X,
	} from '@lucide/svelte';
	import { taskManagerStore } from '$lib/stores/taskManager.svelte';
	import type { TaskType, TrackedTaskInfo } from '$lib/types';
	import * as Sheet from '$lib/ui/sheet';
	import { formatBytes, formatEta, formatSpeed } from '$lib/utils/format';

	let drawerOpen = $state(false);

	/** Get the icon component for a task type */
	function getTaskIcon(taskType: TaskType) {
		switch (taskType) {
			case 'gameDownload':
			case 'contentInstall':
				return Download;
			case 'modpackInstall':
				return Package;
			case 'instanceUpdate':
			case 'launcherUpdate':
				return RefreshCw;
			case 'instanceSetup':
				return Rocket;
			case 'versionMigration':
				return ArrowUpDown;
			case 'instanceImport':
			case 'modpackImport':
				return Import;
			case 'instanceExport':
				return Upload;
			case 'contentScan':
				return Search;
			default:
				return Activity;
		}
	}

	/** Get a human-readable label for a task type */
	function getTaskTypeLabel(taskType: TaskType): string {
		switch (taskType) {
			case 'gameDownload':
				return 'Game Download';
			case 'contentInstall':
				return 'Content Install';
			case 'modpackInstall':
				return 'Modpack Install';
			case 'instanceUpdate':
				return 'Update';
			case 'instanceSetup':
				return 'Instance Setup';
			case 'launcherUpdate':
				return 'Launcher Update';
			case 'versionMigration':
				return 'Version Migration';
			case 'instanceImport':
				return 'Instance Import';
			case 'modpackImport':
				return 'Modpack Import';
			case 'instanceExport':
				return 'Instance Export';
			case 'contentScan':
				return 'Content Scan';
			default:
				return 'Task';
		}
	}

	/** Calculate progress percentage for a task */
	function getProgressPercent(task: TrackedTaskInfo): number {
		if (!task.progress) return 0;
		// Use explicit percent override if available (set by services that compute their own %)
		if (task.progress.percent != null) {
			return Math.min(100, Math.max(0, task.progress.percent));
		}
		if (task.progress.total <= 0) return 0;
		return Math.min(100, Math.max(0, (task.progress.current / task.progress.total) * 100));
	}

	/** Calculate ETA for a task */
	function getTaskEta(task: TrackedTaskInfo): string {
		if (!task.progress?.speedBytesPerSec || task.progress.speedBytesPerSec <= 0) return '';
		if (task.progress.current >= task.progress.total) return '';
		const remaining = task.progress.total - task.progress.current;
		return formatEta(remaining / task.progress.speedBytesPerSec);
	}

	/** Aggregate download speed across all active download tasks */
	const aggregateSpeed = $derived.by(() => {
		let total = 0;
		for (const task of taskManagerStore.activeDownloads) {
			if (task.progress?.speedBytesPerSec) {
				total += task.progress.speedBytesPerSec;
			}
		}
		return total;
	});

	const activeTasks = $derived(taskManagerStore.activeTasks);
	const allTasks = $derived(taskManagerStore.tasks);
	const activeCount = $derived(taskManagerStore.activeCount);
	const hasActiveTasks = $derived(taskManagerStore.hasActiveTasks);

	/** The "primary" task to show in the compact bar (first running download, or first active task) */
	const primaryTask = $derived.by(() => {
		// Prefer a running download-type task
		const dl = taskManagerStore.activeDownloads.find((t) => t.status === 'running');
		if (dl) return dl;
		// Fall back to any running task
		const running = activeTasks.find((t) => t.status === 'running');
		if (running) return running;
		// Fall back to first active
		return activeTasks[0] ?? null;
	});

	const primaryPercent = $derived(primaryTask ? getProgressPercent(primaryTask) : 0);
</script>

<!-- Compact bottom bar: only visible when tasks are active -->
{#if hasActiveTasks}
	<button
		class="border-border bg-card/95 relative z-[60] flex w-full items-center gap-3 border-t px-4 py-2.5 text-left backdrop-blur-sm transition-colors hover:bg-card"
		onclick={() => (drawerOpen = true)}
	>
		<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />

		{#if primaryTask}
			<div class="min-w-0 flex-1">
				<div class="flex items-center gap-2">
					<span class="truncate text-sm font-medium">{primaryTask.label}</span>
					{#if activeCount > 1}
						<span
							class="bg-muted text-muted-foreground flex-shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-medium"
						>
							+{activeCount - 1}
						</span>
					{/if}
				</div>
				<!-- Progress bar -->
				<div class="bg-muted mt-1 h-1 w-full overflow-hidden rounded-full">
					<div
						class="bg-primary h-full transition-all duration-150 ease-out"
						style="width: {primaryPercent}%"
					></div>
				</div>
			</div>

			<div class="flex flex-shrink-0 items-center gap-2">
				{#if aggregateSpeed > 0}
					<span class="text-muted-foreground font-mono text-xs">
						{formatSpeed(aggregateSpeed)}
					</span>
				{/if}
				<span class="text-muted-foreground/70 font-mono text-xs">
					{Math.round(primaryPercent)}%
				</span>
			</div>
		{/if}

		<ChevronUp class="text-muted-foreground h-4 w-4 flex-shrink-0" />
	</button>
{/if}

<!-- Expanding drawer with full task details -->
<Sheet.Root bind:open={drawerOpen}>
	<Sheet.Content side="bottom" class="max-h-[60vh]">
		<Sheet.Header class="pb-2">
			<Sheet.Title class="flex items-center gap-2 text-base">
				<Activity class="h-4 w-4" />
				Tasks
				{#if activeCount > 0}
					<span
						class="bg-primary/10 text-primary rounded-full px-2 py-0.5 text-xs font-medium"
					>
						{activeCount} active
					</span>
				{/if}
			</Sheet.Title>
		</Sheet.Header>

		<div class="flex-1 overflow-y-auto px-4 pb-4">
			{#if allTasks.length === 0}
				<p class="text-muted-foreground py-8 text-center text-sm">No tasks</p>
			{:else}
				<div class="space-y-2">
					{#each allTasks as task (task.id)}
						{@const TaskIcon = getTaskIcon(task.taskType)}
						{@const percent = getProgressPercent(task)}
						{@const eta = getTaskEta(task)}
						{@const isActive = task.status === 'pending' || task.status === 'running'}
						{@const isFailed = task.status === 'failed'}
						{@const isDone = task.status === 'completed' || task.status === 'cancelled'}

						<div
							class="bg-muted/50 rounded-lg p-3 transition-opacity"
							class:opacity-50={isDone}
						>
							<!-- Header row: icon, label, type badge, actions -->
							<div class="flex items-start gap-2">
								{#if task.status === 'running'}
									<Loader2 class="text-primary mt-0.5 h-4 w-4 flex-shrink-0 animate-spin" />
								{:else}
									<TaskIcon
										class="mt-0.5 h-4 w-4 flex-shrink-0 {isFailed
											? 'text-destructive'
											: isDone
												? 'text-muted-foreground/50'
												: 'text-muted-foreground'}"
									/>
								{/if}

								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-2">
										<span
											class="truncate text-sm font-medium"
											class:text-destructive={isFailed}
											title={task.label}
										>
											{task.label}
										</span>
										<span
											class="bg-muted text-muted-foreground flex-shrink-0 rounded px-1.5 py-0.5 text-[10px]"
										>
											{getTaskTypeLabel(task.taskType)}
										</span>
									</div>

									{#if isFailed && task.error}
										<p class="text-destructive mt-0.5 text-xs">{task.error}</p>
									{/if}

									{#if task.progress?.stage}
										<p class="text-muted-foreground mt-0.5 truncate text-xs">
											{task.progress.stage}
											{#if task.progress.currentItem}
												&mdash; {task.progress.currentItem}
											{/if}
										</p>
									{/if}
								</div>

								<!-- Cancel / dismiss button -->
								{#if isActive}
									<button
										class="text-muted-foreground hover:text-destructive mt-0.5 flex-shrink-0 transition-colors"
										onclick={() => taskManagerStore.cancelTask(task.id)}
										title="Cancel task"
									>
										<X class="h-4 w-4" />
									</button>
								{:else}
									<button
										class="text-muted-foreground hover:text-foreground mt-0.5 flex-shrink-0 transition-colors"
										onclick={() => taskManagerStore.dismissTask(task.id)}
										title="Dismiss"
									>
										<X class="h-4 w-4" />
									</button>
								{/if}
							</div>

							<!-- Progress section (only for active tasks) -->
							{#if isActive && task.progress}
								<div class="mt-2">
									<!-- Progress bar -->
									<div class="bg-muted h-1.5 overflow-hidden rounded-full">
										<div
											class="bg-primary h-full transition-all duration-150 ease-out"
											style="width: {percent}%"
										></div>
									</div>

									<!-- Stats row -->
									<div
										class="text-muted-foreground mt-1 flex items-center justify-between text-xs"
									>
										<span>
											{#if task.progress.total > 0}
												{formatBytes(task.progress.current)} / {formatBytes(task.progress.total)}
											{/if}
										</span>
										<div class="flex items-center gap-2">
											{#if task.progress.speedBytesPerSec && task.progress.speedBytesPerSec > 0}
												<span class="flex items-center gap-1 font-mono">
													<Download class="h-3 w-3" />
													{formatSpeed(task.progress.speedBytesPerSec)}
												</span>
											{/if}
											{#if eta}
												<span>{eta}</span>
											{/if}
											<span class="font-mono">{Math.round(percent)}%</span>
										</div>
									</div>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</Sheet.Content>
</Sheet.Root>
