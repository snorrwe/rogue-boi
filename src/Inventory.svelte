<script>
	export let inventory;
	export let core;

	const useItem = (id) => {
		core.useItem(id);
	};
</script>

<div>
	<ul>
		{#each inventory ?? [] as item}
			<li class="item" on:click={() => item.usable && useItem(item.id)}>
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
</style>
