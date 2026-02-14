<script lang="ts">
	import { Check, Copy, ExternalLink, LogOut, Plus, Shirt, Star, UserPlus, WifiOff } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { onMount } from 'svelte';
	import SkinCapeManager from '$lib/components/skin/SkinCapeManager.svelte';
	import SkinFaceThumbnail from '$lib/components/skin/SkinFaceThumbnail.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import type { MinecraftAccount, MinecraftProfile } from '$lib/types';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';

	let copiedCode = $state(false);
	let skinManagerAccount = $state<MinecraftAccount | null>(null);
	let showOfflineForm = $state(false);
	let offlineUsername = $state('');
	let offlineError = $state<string | null>(null);
	let isCreatingOffline = $state(false);


	onMount(() => {
		accountsStore.load();
	});

	// Require at least one Microsoft account before allowing offline account creation
	const hasMicrosoftAccount = $derived(
		accountsStore.accounts.some(a => a.accountType === 'microsoft')
	);

	async function setActive(accountId: string) {
		await accountsStore.setActive(accountId);
	}

	async function logout(accountId: string) {
		const confirmed = await alertDialogStore.confirm({
			title: 'Log Out Account',
			message: 'Are you sure you want to log out this account?',
			type: 'warning',
			confirmText: 'Log Out',
			cancelText: 'Cancel',
		});
		if (confirmed) {
			await accountsStore.deleteAccount(accountId);
		}
	}

	function copyCode() {
		if (accountsStore.deviceCode) {
			navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
			copiedCode = true;
			setTimeout(() => (copiedCode = false), 2000);
		}
	}

	async function copyCodeAndOpen() {
		if (accountsStore.deviceCode) {
			navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
			copiedCode = true;
			setTimeout(() => (copiedCode = false), 2000);
			await openUrl(accountsStore.deviceCode.verificationUri);
		}
	}

	function getAvatarUrl(username: string): string {
		// Use minotar for username-based avatar
		return `https://minotar.net/avatar/${username}/64`;
	}

	async function createOfflineAccount() {
		if (!offlineUsername.trim()) return;
		isCreatingOffline = true;
		offlineError = null;
		try {
			await accountsStore.createOfflineAccount(offlineUsername.trim());
			offlineUsername = '';
			showOfflineForm = false;
		} catch (e: unknown) {
			offlineError = e instanceof Error ? e.message : 'Failed to create offline account';
		} finally {
			isCreatingOffline = false;
		}
	}

	function openSkinManager(account: MinecraftAccount) {
		skinManagerAccount = account;
	}

	function closeSkinManager() {
		skinManagerAccount = null;
	}

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	function handleProfileUpdated(_profile: MinecraftProfile) {
		// Refresh accounts to update skin/cape URLs
		accountsStore.load();
	}

	function handleOfflineSkinUpdated() {
		// Refresh accounts to update skin data URLs
		accountsStore.load();
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl">Accounts</h1>
		<div class="flex gap-2">
			<Button onclick={() => accountsStore.startAuth()} disabled={accountsStore.isAuthenticating}>
				<UserPlus class="mr-2 h-4 w-4" />
				Microsoft Account
			</Button>
			<Button
				variant="outline"
				onclick={() => { showOfflineForm = !showOfflineForm; }}
				disabled={!hasMicrosoftAccount}
				title={!hasMicrosoftAccount ? 'Log in with a Microsoft account first' : ''}
			>
				<WifiOff class="mr-2 h-4 w-4" />
				Offline Account
			</Button>
		</div>
	</div>

	<!-- Add Account Options -->
	{#if showOfflineForm}
		<div class="border-border bg-card border-2 p-4 space-y-3">
			<h2 class="text-sm font-bold">Add Offline Account</h2>
			<p class="text-muted-foreground text-xs">Offline accounts can play without Microsoft authentication. Custom skins are supported.</p>
			<div class="flex gap-2">
				<Input
					type="text"
					bind:value={offlineUsername}
					placeholder="Username (3-16 characters)"
					class="flex-1"
					onkeydown={(e) => e.key === 'Enter' && createOfflineAccount()}
					minlength={3}
					maxlength={16}
				/>
				<Button onclick={createOfflineAccount} disabled={isCreatingOffline || offlineUsername.trim().length < 3}>
					<Plus class="mr-1 h-4 w-4" />
					Create
				</Button>
				<Button variant="outline" onclick={() => { showOfflineForm = false; offlineError = null; }}>
					Cancel
				</Button>
			</div>
			{#if offlineError}
				<p class="text-destructive text-xs">{offlineError}</p>
			{/if}
		</div>
	{/if}

	<!-- Device Code Auth Modal -->
	{#if accountsStore.isAuthenticating && accountsStore.deviceCode}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
			<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
				<h2 class="text-lg font-bold">Sign in with Microsoft</h2>
				<p class="text-muted-foreground text-sm">Enter this code at the Microsoft login page:</p>

				<div class="flex items-center gap-2">
					<code
						class="bg-background border-border flex-1 border-2 p-3 text-center font-mono text-xl tracking-widest"
					>
						{accountsStore.deviceCode.userCode}
					</code>
					<Button variant="outline" size="sm" onclick={copyCode} class="shrink-0">
						{#if copiedCode}
							<Check class="h-4 w-4" />
						{:else}
							<Copy class="h-4 w-4" />
						{/if}
					</Button>
				</div>

				<Button class="w-full" onclick={copyCodeAndOpen}>
					<ExternalLink class="mr-2 h-4 w-4" />
					Copy Code & Open Login Page
				</Button>

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

	<!-- Auth Error -->
	{#if accountsStore.authError}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			{accountsStore.authError}
			<button class="ml-2 underline" onclick={() => accountsStore.clearError()}>Dismiss</button>
		</div>
	{/if}

	{#if accountsStore.isLoading}
		<div class="text-muted-foreground">Loading accounts...</div>
	{:else if accountsStore.accounts.length === 0}
		<!-- Empty State -->
		<div class="border-border bg-card/50 border-2 border-dashed p-12 text-center">
			<UserPlus class="text-muted-foreground/50 mx-auto h-12 w-12" />
			<p class="text-muted-foreground mt-4 text-sm">No accounts logged in</p>
			<div class="mt-4 flex justify-center gap-2">
				<Button onclick={() => accountsStore.startAuth()}>
					<UserPlus class="mr-2 h-4 w-4" />
					Microsoft Account
				</Button>
				<Button variant="outline" onclick={() => { showOfflineForm = true; }}>
					<WifiOff class="mr-2 h-4 w-4" />
					Offline Account
				</Button>
			</div>
		</div>
	{:else}
		<!-- Account List -->
		<div class="space-y-3">
			{#each accountsStore.accounts as account (account.id)}
				<div class="border-border bg-card flex items-center gap-4 border-2 p-4">
				{#if account.skinUrl}
					<SkinFaceThumbnail
						url={account.skinUrl}
						alt={account.username}
						class="h-12 w-12"
					/>
				{:else}
					<img
						src={getAvatarUrl(account.username)}
						alt={account.username}
						class="pixelated h-12 w-12"
					/>
				{/if}
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="text-lg font-bold">{account.username}</span>
							{#if account.accountType === 'offline'}
								<span
									class="bg-orange-500/20 text-orange-400 border-orange-500/50 border px-2 py-0.5 text-xs"
								>
									<WifiOff class="inline h-3 w-3 mr-0.5" />
									Offline
								</span>
							{/if}
							{#if account.isActive}
								<span
									class="bg-primary/20 text-primary border-primary/50 border px-2 py-0.5 text-xs"
								>
									Active
								</span>
							{/if}
						</div>
						<span class="text-muted-foreground mt-1 block font-mono text-xs">
							{account.uuid}
						</span>
					</div>
					<div class="flex items-center gap-2">
						<Button variant="outline" size="sm" onclick={() => openSkinManager(account)}>
							<Shirt class="mr-1 h-4 w-4" />
							Manage Skin
						</Button>
						{#if !account.isActive}
							<Button variant="outline" size="sm" onclick={() => setActive(account.id)}>
								<Star class="mr-1 h-4 w-4" />
								Set Active
							</Button>
						{/if}
						<Button
							variant="outline"
							size="sm"
							onclick={() => logout(account.id)}
							class="text-destructive hover:bg-destructive/10"
						>
							<LogOut class="h-4 w-4" />
						</Button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Skin & Cape Manager Modal -->
{#if skinManagerAccount}
	<SkinCapeManager
		account={skinManagerAccount}
		onClose={closeSkinManager}
		onProfileUpdated={handleProfileUpdated}
		onOfflineSkinUpdated={handleOfflineSkinUpdated}
	/>
{/if}
