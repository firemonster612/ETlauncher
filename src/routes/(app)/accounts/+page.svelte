<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/ui/button";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { UserPlus, LogOut, Star, Copy, Check, ExternalLink } from "@lucide/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  let copiedCode = $state(false);

  onMount(() => {
    accountsStore.load();
  });

  async function setActive(accountId: string) {
    await accountsStore.setActive(accountId);
  }

  async function logout(accountId: string) {
    if (confirm("Are you sure you want to log out this account?")) {
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
      <UserPlus class="h-4 w-4 mr-2" />
      Add Account
    </Button>
  </div>

  <!-- Device Code Auth Modal -->
  {#if accountsStore.isAuthenticating && accountsStore.deviceCode}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
        <h2 class="text-lg font-bold">Sign in with Microsoft</h2>
        <p class="text-sm text-muted-foreground">
          Enter this code at the Microsoft login page:
        </p>

        <div class="flex items-center gap-2">
          <code class="flex-1 bg-background p-3 text-center text-xl font-mono border-2 border-border tracking-widest">
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
          <ExternalLink class="h-4 w-4 mr-2" />
          Copy Code & Open Login Page
        </Button>

        <p class="text-xs text-muted-foreground text-center">
          {accountsStore.deviceCode.verificationUri}
        </p>

        <p class="text-xs text-muted-foreground text-center animate-pulse">
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
    <div class="bg-destructive/10 border-2 border-destructive p-4 text-destructive text-sm">
      {accountsStore.authError}
      <button class="underline ml-2" onclick={() => accountsStore.clearError()}>Dismiss</button>
    </div>
  {/if}

  {#if accountsStore.isLoading}
    <div class="text-muted-foreground">Loading accounts...</div>
  {:else if accountsStore.accounts.length === 0}
    <!-- Empty State -->
    <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
      <UserPlus class="mx-auto h-12 w-12 text-muted-foreground/50" />
      <p class="mt-4 text-sm text-muted-foreground">No accounts logged in</p>
      <Button class="mt-4" onclick={() => accountsStore.startAuth()}>
        <UserPlus class="h-4 w-4 mr-2" />
        Add Account
      </Button>
    </div>
  {:else}
    <!-- Account List -->
    <div class="space-y-3">
      {#each accountsStore.accounts as account (account.id)}
        <div class="flex items-center gap-4 p-4 border-2 border-border bg-card">
          <img
            src={getAvatarUrl(account.username)}
            alt={account.username}
            class="w-12 h-12 pixelated"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-bold text-lg">{account.username}</span>
              {#if account.isActive}
                <span class="text-xs px-2 py-0.5 bg-primary/20 text-primary border border-primary/50">
                  Active
                </span>
              {/if}
            </div>
            <span class="text-xs text-muted-foreground font-mono block mt-1">
              {account.uuid}
            </span>
          </div>
          <div class="flex items-center gap-2">
            {#if !account.isActive}
              <Button variant="outline" size="sm" onclick={() => setActive(account.id)}>
                <Star class="h-4 w-4 mr-1" />
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
