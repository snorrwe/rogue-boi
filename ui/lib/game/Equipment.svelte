<script>
	import { icons, coreStore, equipment as equs } from '@rogueBoi/store.js';

	$: equipment = $equs;

	const useItem = (item) => {
		$coreStore.setSelection(item.id);
	};
</script>

<div>
	<h2>Equipment</h2>
	<ul>
		{#if equipment && equipment.armor}
			<li class="item" on:click={() => useItem(equipment.armor)}>
				<div
					title={equipment.armor.description}
					style="--fill-color: {equipment.armor.color || 'white'}"
				>
					{@html $icons[equipment.armor.icon]}
				</div>
			</li>
		{/if}
		{#if equipment && equipment.weapon}
			<li class="item" on:click={() => useItem(equipment.weapon)}>
				<div
					title={equipment.weapon.description}
					style="--fill-color: {equipment.weapon.color || 'white'}"
				>
					{@html $icons[equipment.weapon.icon]}
				</div>
			</li>
		{/if}
	</ul>
</div>

<style>
	ul {
		display: grid;
		grid-template-columns: repeat(2, 2.2em);
		grid-auto-rows: 2.2em;
		list-style: none;
	}

	.item {
		height: 2.2rem;
		width: 2.2rem;
		cursor: pointer;
	}
</style>
