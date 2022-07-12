<script>
	export let selected;
	export let core;

	const useItem = (item) => () => {
		core.useItem(item.id);
	};

	const target = (item) => () => {
		core.setTarget(item.id);
	};
</script>

<div class="selected">
	<img
		class="selected icon"
		alt={selected.tag}
		src="icons/{selected.icon}.svg"
	/>
	<div>
		{selected.description}
	</div>
	{#if selected.hp}
		<div>
			Health: {selected.hp.current} / {selected.hp.max}
		</div>
	{/if}
	{#if selected.melee}
		<div>Melee Power: {selected.melee.power}</div>
		<div>Melee Skill: {selected.melee.skill}</div>
	{/if}
	{#if selected.ranged}
		<div>Ranged Power: {selected.ranged.power}</div>
		<div>Ranged Skill: {selected.ranged.skill}</div>
	{/if}
	{#if selected.usable}
		<button on:click={useItem(selected)}>Use</button>
	{/if}
	{#if selected.targetable}
		<button on:click={target(selected)}>Target</button>
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
