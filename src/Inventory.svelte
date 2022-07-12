<script>
	import { createEventDispatcher } from 'svelte';

	export let inventory;
	export let core;

	const dispatch = createEventDispatcher();

	const useItem = (item) => {
		const response = core.fetchEntity(item.id);
		dispatch('selected', response);
	};
</script>

<div>
	<h2>Inventory</h2>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item" on:click={() => useItem(item)}>
				<div title={item.description}>
					<img src="/icons/{item.icon}.svg" alt={item.description} />
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
