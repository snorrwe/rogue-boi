<script>
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
	import Highlight from './Highlight.svelte';
	import Player from './Player.svelte';

	export let core;

	const grid = writable(core.getGrid());
	const selected = writable(null);
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
		if ($selected && $selected.id) {
			selected.set(core.fetchEntity($selected.id));
		}

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
		<div>
			{#if core != null && $grid != null}
				<Grid grid={$grid} {core} on:selected={(event) => selected.set(event.detail)} />
			{/if}
		</div>
		<div>
			{#if $selected != null}
				<Highlight selected={$selected} {core} />
			{/if}
			<div>
				<Player
					inventory={$inventory}
					log={$grid.log}
					alive={$grid.player != null}
					hp={$grid.player && $grid.player.playerHp}
					pos={$grid.player && $grid.player.playerPosition}
					attack={$grid.player && $grid.player.playerAttack}
					{core}
					on:selected={(event) => selected.set(event.detail)}
				/>
			</div>
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
