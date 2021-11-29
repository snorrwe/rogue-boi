<script>
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
	import Player from './Player.svelte';

	export let core;

	const grid = writable(core.getGrid());
	let last = new Date().getTime();

	document.addEventListener('keydown', (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	});

	const gameLoop = () => {
		const now = new Date().getTime();

		core.tick(now - last);

		grid.set(core.getGrid());
		last = now;

		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<head>
	{#if core != null}
		{#each core.icons() as icon}
			<link rel="prefetch" href="/icons/ffffff/transparent/1x1/{icon}" />
		{/each}
	{/if}
</head>
<main>
	<div class="content">
		{#if core != null && $grid != null}
			<Grid grid={$grid} />
		{/if}
		<Player alive={$grid.playerAlive} hp={$grid.playerHp} pos={$grid.playerPos} />
	</div>
</main>

<style>
	main {
		padding: 1em;
		margin: 0 auto;
	}

	.content {
		display: grid;
		grid-template-columns: repeat(2, max-content);
	}
</style>
