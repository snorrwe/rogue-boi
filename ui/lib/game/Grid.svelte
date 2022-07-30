<script>
	import { onDestroy, onMount } from 'svelte';
	import { canvasStore } from '@rogueBoi/store.js';

	let canvas;
	onMount(() => {
		canvasStore.set(canvas);
	});
	onDestroy(() => {
		canvasStore.set(null);
	});
</script>

<canvas width="600" height="600" bind:this={canvas} />
