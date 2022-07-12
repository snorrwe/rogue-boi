<script>
	import { createEventDispatcher } from 'svelte';

	export let grid;
	export let core;

	const dispatch = createEventDispatcher();

	const onClick = (item) => {
		if (item && item.id) {
			const response = core.fetchEntity(item.id);
			dispatch('selected', response);
		} else {
			dispatch('selected', null);
		}
	};

    const cellSize = `${1+ 10 / grid.grid.dims.x}em`;
    console.log(cellSize, grid.grid.dims);
</script>

<div class="grid" style="--cell-size: {cellSize}; --cols: {grid.grid.dims.x}; --rows: {grid.grid.dims.y}">
	{#each grid.grid.data as item}
		<div class="grid-item" on:click={() => onClick(item)}>
			<div class:grid_visible={item.visible}>
				{#if item.icon && item.explored}
					<img src="icons/{item.icon}.svg" alt={item.type} />
				{:else}
					<div class="floor" />
				{/if}
			</div>
		</div>
	{/each}
</div>

<style>
	.grid {
		display: grid;
		grid-template-columns: repeat(var(--cols), var(--cell-size));
		grid-auto-rows: var(--cell-size);
	}

	.grid-item {
		color: #ff3e00;
		color: darkgray;
		background: darkgray;
	}

	.grid-item .grid_visible {
		color: black;
		background: black;
	}

	.floor {
		height: var(--cell-size);
		width: var(--cell-size);
	}
</style>
