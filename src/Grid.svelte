<script>
	import { onDestroy, onMount } from 'svelte';
	import { canvasStore } from './store.js';

	export let core;

	let canvas;

	const onClick = (event) => {
		let rect = event.target.getBoundingClientRect();
		core.canvasClicked(event.clientX - rect.left, event.clientY - rect.top);
	};

	onMount(() => {
		canvasStore.set(canvas);
	});
	onDestroy(() => {
		canvasStore.set(null);
	});
</script>

<canvas on:click={onClick} width="600" height="600" bind:this={canvas} />

<style></style>
