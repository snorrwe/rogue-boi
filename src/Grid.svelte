<script>
	export let grid;
</script>

<div class="grid" style="--cols: {grid.grid.dims.x}; --rows: {grid.grid.dims.y}">
	{#each grid.grid.data as item}
		<div class="grid-item">
			<div class:grid_visible={item.visible}>
				{#if item.icon && item.explored}
					<img src="/icons/ffffff/transparent/1x1/{item.icon}" alt={item.type} />
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
		grid-template-columns: repeat(var(--cols), 2.2em);
		grid-auto-rows: 2.2em;
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
		height: 2.2rem;
		width: 2.2rem;
	}
</style>
