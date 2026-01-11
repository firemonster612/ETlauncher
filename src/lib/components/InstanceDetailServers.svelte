<script lang="ts">
	import { ArrowLeft, Loader2, Play, Search, Shield } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { Server } from '$lib/types';

	interface Props {
		instanceId: string;
		minecraftVersion: string;
		activeAccountId: string | null;
		onBack: () => void;
	}

	let { instanceId, minecraftVersion, activeAccountId, onBack }: Props = $props();

	let servers = $state<Server[]>([]);
	let isLoading = $state(false);
	let search = $state('');
	let error = $state<string | null>(null);
	let connecting = $state<string | null>(null);
	let lastLoadedId = $state<string | null>(null);

	const supportsQuickPlay = $derived(checkQuickPlaySupport(minecraftVersion));

	$effect(() => {
		if (instanceId && instanceId !== lastLoadedId) {
			loadServers();
		}
	});

	const filteredServers = $derived(
		servers.filter((server) => {
			const query = search.trim().toLowerCase();
			if (!query) return true;
			return server.name.toLowerCase().includes(query) || server.ip.toLowerCase().includes(query);
		})
	);

	async function loadServers() {
		isLoading = true;
		error = null;

		try {
			const response = await instanceDetailService.getInstanceServers(instanceId);
			servers = response.servers;
			lastLoadedId = instanceId;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load servers';
			console.error('Failed to load servers:', e);
		} finally {
			isLoading = false;
		}
	}

	async function handleConnect(server: Server) {
		if (!supportsQuickPlay) {
			error = 'Quick Play is only available on Minecraft 1.20+.';
			return;
		}
		if (!activeAccountId) {
			error = 'Select an active account before connecting.';
			return;
		}

		connecting = server.ip;
		error = null;

		try {
			await instanceDetailService.launchIntoServer(instanceId, activeAccountId, server.ip);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to connect';
			console.error('Failed to quick-connect:', e);
		} finally {
			connecting = null;
		}
	}

	function serverIconSrc(server: Server): string {
		if (server.iconBase64) {
			const hasPrefix = server.iconBase64.startsWith('data:');
			return hasPrefix ? server.iconBase64 : `data:image/png;base64,${server.iconBase64}`;
		}
		return '/icons/entities/creeper/creeper.png';
	}

	function checkQuickPlaySupport(version: string): boolean {
		const parts = version
			.split('.')
			.map((p) => p.replace(/[^0-9].*$/, ''))
			.filter(Boolean)
			.map((p) => parseInt(p, 10))
			.filter((n) => !Number.isNaN(n));
		const major = parts[0] ?? 0;
		const minor = parts[1] ?? 0;
		return major > 1 || (major === 1 && minor >= 20);
	}
</script>

<div class="flex h-full flex-col">
	<div class="border-border flex items-center justify-between border-b px-6 py-4">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="icon" onclick={onBack} aria-label="Back">
				<ArrowLeft class="h-5 w-5" />
			</Button>
			<div>
				<p class="text-muted-foreground text-xs tracking-wide uppercase">Servers</p>
				<h2 class="text-xl font-semibold">Multiplayer</h2>
			</div>
		</div>
		{#if !supportsQuickPlay}
			<p class="text-muted-foreground text-xs">Quick Play requires Minecraft 1.20+</p>
		{/if}
	</div>

	<div class="flex flex-1 flex-col gap-4 overflow-y-auto p-6">
		<div class="flex flex-col gap-3 md:flex-row md:items-center">
			<div class="relative flex-1">
				<Input
					placeholder="Search servers..."
					value={search}
					oninput={(e) => (search = e.currentTarget.value)}
					class="pl-10"
				/>
				<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
			</div>
		</div>

		{#if error}
			<div
				class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
			>
				{error}
			</div>
		{/if}

		{#if isLoading}
			<div class="text-muted-foreground flex items-center gap-3">
				<Loader2 class="h-5 w-5 animate-spin" />
				<span>Loading servers...</span>
			</div>
		{:else if filteredServers.length === 0}
			<p class="text-muted-foreground text-sm">No servers saved.</p>
		{:else}
			<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
				{#each filteredServers as server (server.ip)}
					<div class="border-border bg-background/80 flex flex-col gap-3 rounded-lg border p-3">
						<div class="flex items-start gap-3">
							<img
								src={serverIconSrc(server)}
								alt={`${server.name} icon`}
								class="border-border bg-muted h-12 w-12 rounded border object-cover"
							/>
							<div class="min-w-0 flex-1">
								<p class="truncate font-semibold">{server.name}</p>
								<p class="text-muted-foreground truncate text-xs">{server.ip}</p>
								<div class="mt-1 flex flex-wrap items-center gap-2">
									{#if server.hidden}
										<span
											class="bg-muted/60 flex items-center gap-1 rounded border px-2 py-0.5 text-[11px]"
										>
											<Shield class="h-3 w-3" />
											Hidden
										</span>
									{/if}
									{#if server.acceptTextures}
										<span
											class="rounded border bg-emerald-500/20 px-2 py-0.5 text-[11px] text-emerald-500"
										>
											Server textures
										</span>
									{/if}
								</div>
							</div>
						</div>

						{#if supportsQuickPlay}
							<Button
								variant="secondary"
								size="sm"
								class="justify-center"
								onclick={() => handleConnect(server)}
								disabled={!!connecting}
							>
								{#if connecting === server.ip}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
									Connecting...
								{:else}
									<Play class="mr-2 h-4 w-4" />
									Connect
								{/if}
							</Button>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
