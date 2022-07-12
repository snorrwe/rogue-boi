<script>
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
	import Highlight from './Highlight.svelte';
	import Log from './Log.svelte';
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
			<link rel="preload" href="/icons/{icon}.svg" as="image" />
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
					alive={$grid.player != null}
					hp={$grid.player && $grid.player.playerHp}
					pos={$grid.player && $grid.player.playerPosition}
					attack={$grid.player && $grid.player.playerAttack}
					{core}
					on:selected={(event) => selected.set(event.detail)}
				/>
			</div>
		</div>
		<div class="log">
			<h2>Logs</h2>
			<Log log={$grid.log} />
		</div>
	</div>
</main>

<style>
	main {
		padding: 2em;
		margin: 0 auto;
		color: white;
		min-width: 0px;
	}

	.content {
		display: grid;
		grid-template-columns: minmax(0, 2fr) repeat(2, 1fr);
	}

	.log {
		max-height: 100%;
		overflow: auto;
	}
</style>
