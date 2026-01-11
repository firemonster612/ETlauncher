<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, X } from '@lucide/svelte';

	const appWindow = getCurrentWindow();

	function minimize() {
		appWindow.minimize();
	}

	function toggleMaximize() {
		appWindow.toggleMaximize();
	}

	function close() {
		appWindow.close();
	}

	function startDrag(e: MouseEvent) {
		// Only start drag on left click and not on buttons
		if (e.button === 0) {
			appWindow.startDragging();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="border-border bg-sidebar flex h-9 items-center justify-between border-b-2 select-none"
	onmousedown={startDrag}
>
	<div class="px-4">
		<span class="text-xs font-bold tracking-widest uppercase">ETLauncher</span>
	</div>

	<div class="flex items-center">
		<button
			onmousedown={(e) => e.stopPropagation()}
			onclick={minimize}
			class="text-muted-foreground hover:bg-primary/20 hover:text-primary flex h-9 w-10 items-center justify-center transition-all"
			aria-label="Minimize"
		>
			<Minus class="h-4 w-4" strokeWidth={3} />
		</button>
		<button
			onmousedown={(e) => e.stopPropagation()}
			onclick={toggleMaximize}
			class="text-muted-foreground hover:bg-primary/20 hover:text-primary flex h-9 w-10 items-center justify-center transition-all"
			aria-label="Maximize"
		>
			<Square class="h-3.5 w-3.5" strokeWidth={3} />
		</button>
		<button
			onmousedown={(e) => e.stopPropagation()}
			onclick={close}
			class="text-muted-foreground hover:bg-destructive flex h-9 w-10 items-center justify-center transition-all hover:text-white"
			aria-label="Close"
		>
			<X class="h-4 w-4" strokeWidth={3} />
		</button>
	</div>
</div>
