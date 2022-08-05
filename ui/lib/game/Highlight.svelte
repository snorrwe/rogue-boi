<script>
	import { iconStore } from '@rogueBoi/store.js';

	export let selected;
	export let core;
	export let targetingMode;

	const useItem = (item) => () => {
		core.useItem(item.id);
	};

	const dropItem = (item) => () => {
		core.dropItem(item.id);
	};

	const target = (item) => () => {
		core.setTarget(item.id);
	};
</script>

<div class="icon" style="--fill-color: {selected.color || 'white'}">
	{@html $iconStore[selected.icon]}
</div>
{#if selected.name}
	<h3>
		{selected.name}
	</h3>
{/if}
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
{#if selected.equipable}
	<button on:click={useItem(selected)}>Equip</button>
{/if}
{#if selected.item}
	<button on:click={dropItem(selected)}>Drop</button>
{/if}
{#if selected.targetable && targetingMode}
	<button on:click={target(selected)}>Target</button>
{/if}

<style>
	.icon {
		height: 3.2rem;
		width: 3.2rem;
	}
</style>
