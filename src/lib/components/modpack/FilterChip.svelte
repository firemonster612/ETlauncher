<script lang="ts">
	import { X } from '@lucide/svelte';

	interface Props {
		label: string;
		onRemove: () => void;
	}

	let { label, onRemove }: Props = $props();
</script>

<button type="button" class="filter-chip" onclick={onRemove}>
	<span class="capitalize">{label}</span>
	<span class="filter-chip-remove">
		<X class="h-3 w-3" />
	</span>
</button>
