<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { Button } from '$lib/ui/button';
	import { ChevronUp, LogIn, Settings, Copy, Check, ExternalLink } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	let isOpen = $state(false);
	let dropdownRef = $state<HTMLDivElement | null>(null);
	let codeCopied = $state(false);
	let avatarError = $state(false);

	onMount(() => {
		accountsStore.load();

		function handleClickOutside(event: MouseEvent) {
			if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
				isOpen = false;
			}
		}

		document.addEventListener('click', handleClickOutside);
		return () => document.removeEventListener('click', handleClickOutside);
	});

	function getAvatarUrl(username: string): string {
		return `https://minotar.net/avatar/${username}/32`;
	}

	function handleAvatarError() {
		console.error('Avatar failed to load for:', accountsStore.activeAccount?.username);
		avatarError = true;
	}

	async function switchAccount(accountId: string) {
		await accountsStore.setActive(accountId);
		isOpen = false;
	}

	function goToAccounts() {
		isOpen = false;
		goto(resolve('/accounts'));
	}

	function startLogin() {
		isOpen = false;
		accountsStore.startAuth();
	}

	function copyCode() {
		if (accountsStore.deviceCode) {
			navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
			codeCopied = true;
			setTimeout(() => (codeCopied = false), 2000);
		}
	}

	async function copyCodeAndOpen() {
		if (accountsStore.deviceCode) {
			navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
			codeCopied = true;
			setTimeout(() => (codeCopied = false), 2000);
			await openUrl(accountsStore.deviceCode.verificationUri);
		}
	}
</script>

<div class="relative" bind:this={dropdownRef}>
	{#if accountsStore.activeAccount}
		<!-- Active Account Button -->
		<button
			class="hover:bg-sidebar-accent flex w-full items-center gap-2 rounded-md p-2 text-left transition-colors"
			onclick={() => (isOpen = !isOpen)}
		>
			{#if avatarError}
				<div
					class="bg-primary/20 flex h-8 w-8 items-center justify-center rounded text-xs font-bold"
				>
					{accountsStore.activeAccount.username.charAt(0).toUpperCase()}
				</div>
			{:else}
				<img
					src={getAvatarUrl(accountsStore.activeAccount.username)}
					alt={accountsStore.activeAccount.username}
					class="pixelated h-8 w-8 rounded"
					onerror={handleAvatarError}
				/>
			{/if}
			<div class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
				<p class="truncate text-sm font-medium">{accountsStore.activeAccount.username}</p>
			</div>
			<ChevronUp
				class="text-muted-foreground h-4 w-4 transition-transform group-data-[collapsible=icon]:hidden {isOpen
					? ''
					: 'rotate-180'}"
			/>
		</button>
	{:else}
		<!-- No Account - Login Button -->
		<Button variant="outline" class="w-full justify-start" onclick={startLogin}>
			<LogIn class="mr-2 h-4 w-4" />
			<span class="group-data-[collapsible=icon]:hidden">Login</span>
		</Button>
	{/if}

	<!-- Dropdown Menu -->
	{#if isOpen && accountsStore.accounts.length > 0}
		<div
			class="bg-popover border-border absolute right-0 bottom-full left-0 z-50 mb-1 overflow-hidden rounded-md border shadow-lg"
		>
			<!-- Other Accounts -->
			{#each accountsStore.accounts as account (account.id)}
				{#if account.id !== accountsStore.activeAccount?.id}
					<button
						class="hover:bg-accent flex w-full items-center gap-2 p-2 text-left transition-colors"
						onclick={() => switchAccount(account.id)}
					>
						<img
							src={getAvatarUrl(account.username)}
							alt={account.username}
							class="pixelated h-6 w-6 rounded"
							onerror={(e) => {
								const target = e.currentTarget as HTMLImageElement;
								target.style.display = 'none';
								const fallback = document.createElement('div');
								fallback.className =
									'w-6 h-6 rounded bg-primary/20 flex items-center justify-center text-xs font-bold';
								fallback.textContent = account.username.charAt(0).toUpperCase();
								target.parentElement?.insertBefore(fallback, target);
							}}
						/>
						<span class="truncate text-sm">{account.username}</span>
					</button>
				{/if}
			{/each}

			<!-- Divider -->
			<div class="border-border border-t"></div>

			<!-- Manage Accounts Link -->
			<button
				class="hover:bg-accent flex w-full items-center gap-2 p-2 text-left text-sm transition-colors"
				onclick={goToAccounts}
			>
				<Settings class="h-4 w-4" />
				<span>Manage Accounts</span>
			</button>
		</div>
	{/if}
</div>

<!-- Device Code Modal (when authenticating from sidebar) -->
{#if accountsStore.isAuthenticating && accountsStore.deviceCode}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-lg font-bold">Sign in with Microsoft</h2>
			<p class="text-muted-foreground text-sm">Enter this code at the Microsoft login page:</p>

			<!-- Code display with copy button -->
			<div class="flex items-center gap-2">
				<code
					class="bg-background border-border flex-1 border-2 p-3 text-center font-mono text-xl tracking-widest"
				>
					{accountsStore.deviceCode.userCode}
				</code>
				<Button variant="outline" size="sm" onclick={copyCode} class="shrink-0">
					{#if codeCopied}
						<Check class="h-4 w-4" />
					{:else}
						<Copy class="h-4 w-4" />
					{/if}
				</Button>
			</div>

			<!-- Primary action: Copy code and open URL -->
			<Button class="w-full" onclick={copyCodeAndOpen}>
				<ExternalLink class="mr-2 h-4 w-4" />
				Copy Code & Open Login Page
			</Button>

			<!-- URL display -->
			<p class="text-muted-foreground text-center text-xs">
				{accountsStore.deviceCode.verificationUri}
			</p>

			<p class="text-muted-foreground animate-pulse text-center text-xs">
				Waiting for authentication...
			</p>

			<Button variant="outline" class="w-full" onclick={() => accountsStore.stopAuth()}>
				Cancel
			</Button>
		</div>
	</div>
{/if}

<!-- Auth Error Modal -->
{#if accountsStore.authError}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-destructive text-lg font-bold">Authentication Error</h2>
			<p class="text-sm">{accountsStore.authError}</p>
			<Button variant="outline" class="w-full" onclick={() => accountsStore.clearError()}>
				Close
			</Button>
		</div>
	</div>
{/if}
