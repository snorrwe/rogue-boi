<script>
	import { coreStore } from '@rogueBoi/store.js';

	let core;
	let dungeon;
	let tiles = [];

	const dims = [64, 64]; // FIXME: query from core, or set on dungeon generation

	coreStore.subscribe((c) => {
		core = c;
		regenerate(dims);
	});
	// Debug ui for dungeon generation
	function regenerate(dims) {
		// TODO: pass args
		dungeon = core.generate_dungeon();
		tiles.length = 0;
		for (let y = 0; y < dims[1]; ++y) {
			for (let x = 0; x < dims[0]; ++x) {
				tiles.push(dungeon.get(`${x};${y}`));
			}
		}
	}
</script>

Debug tool to visualize the map generator's behaviour
<button on:click={() => regenerate(dims)}>Regen</button>
<div class="gridContainer">
	<div class="grid">
		{#each tiles as tile}
			{#if tile}
				<embed type="image/svg+xml" src={`icons/${tile}.svg`} />
			{:else}
				<div />
			{/if}
		{/each}
	</div>
</div>

<style>
	.gridContainer {
		width: 100%;
		height: 60%;
		overflow: auto;
	}

	.grid {
		max-width: 100%;
		max-height: 100%;
		display: grid;
		grid-template-columns: repeat(64, 1em);
		grid-auto-rows: 1em;
	}
</style>
