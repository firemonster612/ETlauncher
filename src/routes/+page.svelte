<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';

	onMount(() => {
		goto(resolve('/instances'));
	});
</script>

<div class="flex h-full items-center justify-center">
	<p class="text-muted-foreground">Redirecting...</p>
</div>
