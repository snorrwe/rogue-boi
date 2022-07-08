<script>
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
	import Inventory from './Inventory.svelte';
	import Player from './Player.svelte';

	export let core;

	const grid = writable(core.getGrid());
	const inventory = writable(core.getInventory());
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
		inventory.set(core.getInventory());
		last = now;

		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<svelte:head>
	{#if core != null}
		{#each core.icons() as icon}
			<link rel="preload" href="/icons/ffffff/transparent/1x1/{icon}" as="image" />
		{/each}
	{/if}
</svelte:head>
<main>
	<div class="content">
		{#if core != null && $grid != null}
			<Grid grid={$grid} />
		{/if}
		<div>
			<Inventory inventory={$inventory} />
			<Player log={$grid.log} alive={$grid.playerAlive} hp={$grid.playerHp} pos={$grid.playerPos} />
		</div>
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
