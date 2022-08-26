<script>
	import { icons, coreStore, inventory as invs } from '@rogueBoi/store.js';

	$: inventory = $invs;

	const useItem = (item) => {
		$coreStore.setSelection(item.id);
	};
</script>

<div>
	<h2>Inventory</h2>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item" on:click={() => useItem(item)}>
				<div title={item.description} style="--fill-color: {item.color || 'white'}">
					{@html $icons[item.icon]}
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
