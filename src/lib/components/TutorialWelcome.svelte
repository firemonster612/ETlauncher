<script lang="ts">
	import { Gamepad2, ArrowRight, X } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { tutorialStore } from '$lib/stores/tutorial.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';

	interface Props {
		open: boolean;
	}

	let { open }: Props = $props();

	function handleStart() {
		tutorialStore.start();
	}

	async function handleSkip() {
		tutorialStore.hideWelcome();
		await settingsStore.update({ setupCompleted: true });
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) handleSkip();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') handleSkip();
	}

	function stopClickPropagation(e: MouseEvent) {
		e.stopPropagation();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
	<div
		class="fixed top-[var(--titlebar-height)] right-0 bottom-0 left-0 z-50 flex items-center justify-center bg-black/50 p-6"
		onclick={handleBackdropClick}
		role="dialog"
		aria-label="Welcome to ETLauncher"
	>
		<div
			class="bg-card border-border relative flex max-h-[calc(100vh-var(--titlebar-height)-4rem)] w-full max-w-lg flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
			onclick={stopClickPropagation}
			role="presentation"
			tabindex="-1"
		>
			<div class="border-border flex items-center justify-between border-b px-6 py-4">
				<h3 class="text-2xl font-semibold">Welcome to ETLauncher</h3>
				<Button variant="secondary" size="icon" onclick={handleSkip} aria-label="Close">
					<X class="h-5 w-5" />
				</Button>
			</div>

			<div class="space-y-6 overflow-y-auto px-6 py-6">
				<!-- App Icon/Logo -->
				<div class="bg-primary/10 mx-auto flex h-20 w-20 items-center justify-center rounded-2xl">
					<Gamepad2 class="text-primary h-10 w-10" />
				</div>

				<p class="text-center text-base">
					Your all-in-one Minecraft launcher for managing instances, modpacks, and more.
				</p>

				<!-- What you'll learn section -->
				<div class="space-y-3">
					<h4 class="text-foreground text-sm font-medium">What you'll learn:</h4>
					<ul class="text-muted-foreground space-y-2 text-sm">
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">1</span>
							</div>
							<span>How to sign in with your Microsoft account</span>
						</li>
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">2</span>
							</div>
							<span>How to browse and install modpacks</span>
						</li>
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">3</span>
							</div>
							<span>How to create custom instances from scratch</span>
						</li>
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">4</span>
							</div>
							<span>How to add mods, shaders, and resource packs</span>
						</li>
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">5</span>
							</div>
							<span>How to update Minecraft versions and mod loaders</span>
						</li>
						<li class="flex items-start gap-2">
							<div
								class="bg-primary/10 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full"
							>
								<span class="text-primary text-xs font-medium">6</span>
							</div>
							<span>How to launch and manage your Minecraft installations</span>
						</li>
					</ul>
				</div>

				<p class="text-muted-foreground text-sm">
					This quick tutorial will walk you through both ways to play: installing ready-made
					modpacks and creating your own custom instances.
				</p>
			</div>

			<div class="border-border flex flex-col gap-2 border-t px-6 pt-4 sm:flex-row">
				<Button variant="ghost" onclick={handleSkip} class="w-full sm:w-auto">Skip for now</Button>
				<Button onclick={handleStart} class="w-full gap-2 sm:w-auto">
					Start Tutorial
					<ArrowRight class="h-4 w-4" />
				</Button>
			</div>
		</div>
	</div>
{/if}
