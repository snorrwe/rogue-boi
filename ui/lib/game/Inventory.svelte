<script>
	import { iconStore } from '@rogueBoi/store.js';

	export let inventory;
	export let core;

	const useItem = (item) => {
		core.setSelection(item.id);
	};
</script>

<div>
	<h2>Inventory</h2>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item" on:click={() => useItem(item)}>
				<div title={item.description} style="--fill-color: {item.color || 'white'}">
					{@html $iconStore[item.icon]}
				</div>
			</li>
		{/each}
	</ul>
</div>

<style>
	div {
		color: white;
	}

	.item {
		height: 2.2rem;
		width: 2.2rem;
		cursor: pointer;
	}

	ul {
		display: grid;
		grid-template-columns: repeat(6, 2.2em);
		grid-auto-rows: 2.2em;
		list-style: none;
	}
</style>
