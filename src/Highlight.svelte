<script>
	export let selected;
	export let core;

	const useItem = (item) => () => {
		core.useItem(item.id);
	};
</script>

<div class="selected">
	<img
		class="selected icon"
		alt={selected.tag}
		src="/icons/ffffff/transparent/1x1/{selected.icon}"
	/>
	<div>
		{selected.description}
	</div>
	{#if selected.hp}
		<div>
			Health: {selected.hp.current} / {selected.hp.max}
		</div>
	{/if}
	{#if selected.usable}
		<button on:click={useItem(selected)}>Use</button>
	{/if}
</div>

<style>
	.selected {
		color: white;
	}
	.selected.icon {
		height: 3.2rem;
	}
</style>
