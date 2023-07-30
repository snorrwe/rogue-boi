<script>
	import { coreStore, icons as iconsSvg } from '@rogueBoi/store.js';

	let core;
	let dungeon;
	let tiles = [];
	let icons;
	let level = 1;

	iconsSvg.subscribe((i) => {
		icons = {};
		for (const [k, v] of Object.entries(i)) {
			icons[k] = `data:image/svg+xml;base64,${btoa(v)}`;
		}
	});

	const dims = [64, 64]; // FIXME: query from core, or set on dungeon generation

	coreStore.subscribe((c) => {
		core = c;
		regenerate({ dims, level });
	});
	// Debug ui for dungeon generation
	function regenerate({ dims, level }) {
		dungeon = core.generate_dungeon({ level, dims: { x: dims[0], y: dims[1] } });
		tiles.length = 0;
		for (let y = 0; y < dims[1]; ++y) {
			for (let x = 0; x < dims[0]; ++x) {
				tiles.push(dungeon.get(`${x};${y}`));
			}
		}
	}
</script>

Debug tool to visualize the map generator's behaviour
<input type="number" placeholder="level" min="1" bind:value={level} />
<button on:click={() => regenerate({ dims, level })}>Regen</button>
<div class="gridContainer">
	<div class="grid">
		{#each tiles as tile}
			{#if tile}
				<img src={icons[tile]} alt={tile} />
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
