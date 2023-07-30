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

	let dims = [64, 64]; // FIXME: query from core, or set on dungeon generation
	let desiredDims = dims;

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

<div>Debug tool to visualize the map generator's behaviour</div>
<div>
	<div>
		<label for="level">Level</label>
		<input type="number" name="level" placeholder="level" min="1" bind:value={level} />
	</div>
	<div>
		<label for="x">Width</label>
		<input type="number" name="x" placeholder="x" min="1" bind:value={desiredDims[0]} />
		<label for="y">Height</label>
		<input type="number" name="y" placeholder="y" min="1" bind:value={desiredDims[1]} />
	</div>
	<div>
		<button
			on:click={() => {
				dims = desiredDims;
				regenerate({ dims, level });
			}}>Regen</button
		>
	</div>
	<div class="gridContainer">
		<div class="grid" style="--cols:{dims[0]}">
			{#each tiles as tile}
				{#if tile}
					<img src={icons[tile]} alt={tile} />
				{:else}
					<div />
				{/if}
			{/each}
		</div>
	</div>
</div>

<style>
	.gridContainer {
		width: 100vw;
		height: 80vh;
		overflow: auto;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(var(--cols), 1em);
		grid-auto-rows: 1em;
	}
</style>
