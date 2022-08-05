<script>
	import { createEventDispatcher } from 'svelte';

	import Inventory from './Inventory.svelte';
	const dispatch = createEventDispatcher();

	export let alive;
	export let hp;
	export let pos;
	export let attack;
	export let inventory;
	export let core;
	export let targeting;
	export let currentXp;
	export let neededXp;
	export let level;
	export let levelup;
	export let defense;
</script>

<div>
	{#if alive}
		<Inventory {inventory} {core} on:selected={(ev) => dispatch('selected', ev.detail)} />

		{#if targeting}
			<button on:click={() => core.cancelItemUse()}>Cancel item use</button>
		{/if}

		<h2>Player stats</h2>
		{#if hp != null}
			<p id="player-hp">
				{#if levelup}
					<button on:click={() => core.setLevelupStat({ ty: 'Hp' })}>+</button>
				{/if}
				Health: {hp.current} / {hp.max}
			</p>
		{/if}
		{#if attack != null}
			<p>
				{#if levelup}
					<button on:click={() => core.setLevelupStat({ ty: 'Attack' })}>+</button>
				{/if}
				Attack Power: {attack}
			</p>
		{/if}
		{#if defense != null}
			<p>
				{#if levelup}
					<button on:click={() => core.setLevelupStat({ ty: 'MeleeDefense' })}>+</button>
				{/if}
				Melee Defense: {defense.meleeDefense}
			</p>
		{/if}
		{#if pos != null}
			<p id="player-pos">
				Position: {pos.x}, {pos.y}
			</p>
		{/if}
		{#if level != null}
			<p>Level: {level}</p>
		{/if}
		{#if currentXp != null}
			<p>Experience: {currentXp} / {neededXp}</p>
		{/if}
		<button on:click={() => core.wait()}>Wait</button>
	{/if}

	{#if !alive}
		<p>You died!</p>
		<button on:click={() => core.restart()}>Restart</button>
	{/if}
</div>

<style></style>
