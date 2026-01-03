<script lang="ts">
  import { goto } from "$app/navigation";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { Button } from "$lib/ui/button";
  import { ChevronUp, LogIn, Settings, Copy, Check, ExternalLink } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

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

    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });

  function getAvatarUrl(username: string): string {
    return `https://minotar.net/avatar/${username}/32`;
  }

  function handleAvatarError() {
    console.error("Avatar failed to load for:", accountsStore.activeAccount?.username);
    avatarError = true;
  }

  async function switchAccount(accountId: string) {
    await accountsStore.setActive(accountId);
    isOpen = false;
  }

  function goToAccounts() {
    isOpen = false;
    goto("/accounts");
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
      class="w-full flex items-center gap-2 p-2 rounded-md hover:bg-sidebar-accent transition-colors text-left"
      onclick={() => (isOpen = !isOpen)}
    >
      {#if avatarError}
        <div class="w-8 h-8 rounded bg-primary/20 flex items-center justify-center text-xs font-bold">
          {accountsStore.activeAccount.username.charAt(0).toUpperCase()}
        </div>
      {:else}
        <img
          src={getAvatarUrl(accountsStore.activeAccount.username)}
          alt={accountsStore.activeAccount.username}
          class="w-8 h-8 rounded pixelated"
          onerror={handleAvatarError}
        />
      {/if}
      <div class="flex-1 min-w-0 group-data-[collapsible=icon]:hidden">
        <p class="text-sm font-medium truncate">{accountsStore.activeAccount.username}</p>
      </div>
      <ChevronUp
        class="h-4 w-4 text-muted-foreground transition-transform group-data-[collapsible=icon]:hidden {isOpen
          ? ''
          : 'rotate-180'}"
      />
    </button>
  {:else}
    <!-- No Account - Login Button -->
    <Button variant="outline" class="w-full justify-start" onclick={startLogin}>
      <LogIn class="h-4 w-4 mr-2" />
      <span class="group-data-[collapsible=icon]:hidden">Login</span>
    </Button>
  {/if}

  <!-- Dropdown Menu -->
  {#if isOpen && accountsStore.accounts.length > 0}
    <div
      class="absolute bottom-full left-0 right-0 mb-1 bg-popover border border-border rounded-md shadow-lg overflow-hidden z-50"
    >
      <!-- Other Accounts -->
      {#each accountsStore.accounts as account (account.id)}
        {#if account.id !== accountsStore.activeAccount?.id}
          <button
            class="w-full flex items-center gap-2 p-2 hover:bg-accent transition-colors text-left"
            onclick={() => switchAccount(account.id)}
          >
            <img
              src={getAvatarUrl(account.username)}
              alt={account.username}
              class="w-6 h-6 rounded pixelated"
            />
            <span class="text-sm truncate">{account.username}</span>
          </button>
        {/if}
      {/each}

      <!-- Divider -->
      <div class="border-t border-border"></div>

      <!-- Manage Accounts Link -->
      <button
        class="w-full flex items-center gap-2 p-2 hover:bg-accent transition-colors text-left text-sm"
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
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold">Sign in with Microsoft</h2>
      <p class="text-sm text-muted-foreground">
        Enter this code at the Microsoft login page:
      </p>

      <!-- Code display with copy button -->
      <div class="flex items-center gap-2">
        <code class="flex-1 bg-background p-3 text-center text-xl font-mono border-2 border-border tracking-widest">
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
        <ExternalLink class="h-4 w-4 mr-2" />
        Copy Code & Open Login Page
      </Button>

      <!-- URL display -->
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

<!-- Auth Error Modal -->
{#if accountsStore.authError}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold text-destructive">Authentication Error</h2>
      <p class="text-sm">{accountsStore.authError}</p>
      <Button variant="outline" class="w-full" onclick={() => accountsStore.clearError()}>
        Close
      </Button>
    </div>
  </div>
{/if}
