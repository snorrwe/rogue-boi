<script>
	export let inventory;
	export let core;

	const useItem = (id) => {
		core.useItem(id);
	};
</script>

<div>
	<h2>Inventory</h2>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item" class:usable={item.usable} on:click={() => item.usable && useItem(item.id)}>
				<div title={item.description}>
					<img src="/icons/ffffff/transparent/1x1/{item.icon}" />
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
	}

	.usable {
		cursor: pointer;
	}

	ul {
		display: grid;
		grid-template-columns: repeat(6, 2.2em);
		grid-auto-rows: 2.2em;
	}
</style>
