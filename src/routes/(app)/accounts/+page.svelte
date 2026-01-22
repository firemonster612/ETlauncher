<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/ui/button';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { UserPlus, LogOut, Star, Copy, Check, ExternalLink } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	let copiedCode = $state(false);

	onMount(() => {
		accountsStore.load();
	});

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
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl">Accounts</h1>
		<Button onclick={() => accountsStore.startAuth()} disabled={accountsStore.isAuthenticating}>
			<UserPlus class="mr-2 h-4 w-4" />
			Add Account
		</Button>
	</div>

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
			<Button class="mt-4" onclick={() => accountsStore.startAuth()}>
				<UserPlus class="mr-2 h-4 w-4" />
				Add Account
			</Button>
		</div>
	{:else}
		<!-- Account List -->
		<div class="space-y-3">
			{#each accountsStore.accounts as account (account.id)}
				<div class="border-border bg-card flex items-center gap-4 border-2 p-4">
					<img
						src={getAvatarUrl(account.username)}
						alt={account.username}
						class="pixelated h-12 w-12"
					/>
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<span class="text-lg font-bold">{account.username}</span>
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
