<script>
	import { onDestroy, onMount } from 'svelte';
	import { canvasStore } from './store.js';

	export let core;

	let canvas;

	const onClick = (event) => {
		// TODO: onCanvasClick in core
	};

	onMount(() => {
		canvasStore.set(canvas);
	});
	onDestroy(() => {
		canvasStore.set(null);
	});
</script>

<canvas on:click={onClick} width="800" height="800" bind:this={canvas} />

<style></style>
