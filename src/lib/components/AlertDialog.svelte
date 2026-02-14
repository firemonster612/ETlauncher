<script lang="ts">
	import { AlertTriangle, CheckCircle, Info, XCircle } from '@lucide/svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { Button } from '$lib/ui/button';

	const iconMap = {
		info: Info,
		success: CheckCircle,
		warning: AlertTriangle,
		error: XCircle,
	};

	const iconColorMap = {
		info: 'text-primary',
		success: 'text-green-500',
		warning: 'text-yellow-500',
		error: 'text-destructive',
	};

	let Icon = $derived(iconMap[alertDialogStore.type]);
	let iconColor = $derived(iconColorMap[alertDialogStore.type]);

	function handleKeydown(e: KeyboardEvent) {
		if (!alertDialogStore.isOpen) return;

		if (e.key === 'Escape') {
			if (alertDialogStore.cancelText) {
				alertDialogStore.handleCancel();
			} else {
				alertDialogStore.handleConfirm();
			}
		} else if (e.key === 'Enter') {
			alertDialogStore.handleConfirm();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if alertDialogStore.isOpen}
	<div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50">
		<div
			class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6"
			data-alert-dialog
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="presentation"
		>
			<div class="flex items-start gap-4">
				<div class={iconColor}>
					<Icon class="h-6 w-6 flex-shrink-0" />
				</div>
				<div class="min-w-0 flex-1 space-y-2">
					{#if alertDialogStore.title}
						<h2 class="text-lg font-bold">{alertDialogStore.title}</h2>
					{/if}
					<p class="text-muted-foreground text-sm">{alertDialogStore.message}</p>
				</div>
			</div>

			<div class="flex justify-end gap-2 pt-2">
				{#if alertDialogStore.cancelText}
					<Button variant="outline" onclick={() => alertDialogStore.handleCancel()}>
						{alertDialogStore.cancelText}
					</Button>
				{/if}
				<Button
					variant={alertDialogStore.type === 'error' ? 'destructive' : 'default'}
					onclick={() => alertDialogStore.handleConfirm()}
				>
					{alertDialogStore.confirmText}
				</Button>
			</div>
		</div>
	</div>
{/if}
