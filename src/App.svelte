<script>
	import { writable } from 'svelte/store';
	import Grid from './Grid.svelte';
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
		if ($selected?.id) {
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
				<div class="selected">
					<img
						class="selected icon"
						alt={$selected.tag}
						src="/icons/ffffff/transparent/1x1/{$selected.icon}"
					/>
					<div>
						{$selected.description}
					</div>
					{#if $selected.hp}
						<div>
							Health: {$selected.hp.current} / {$selected.hp.max}
						</div>
					{/if}
				</div>
			{/if}
			<div>
				<Player
					inventory={$inventory}
					log={$grid.log}
					alive={$grid.player != null}
					hp={$grid.player?.playerHp}
					pos={$grid.player?.playerPosition}
					attack={$grid.player?.playerAttack}
					{core}
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

	.selected {
		color: white;
	}
	.selected.icon {
		height: 3.2rem;
	}
</style>
