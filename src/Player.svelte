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
</script>

<div>
	{#if alive}
		<Inventory {inventory} {core} on:selected={(ev) => dispatch('selected', ev.detail)} />

		<h2>Player stats</h2>
		{#if hp != null}
			<p id="player-hp">
				Health: {hp.current} / {hp.max}
			</p>
		{/if}
		{#if attack != null}
			<p id="player-attack">
				Attack Power: {attack}
			</p>
		{/if}
		{#if pos != null}
			<p id="player-pos">
				Position: {pos.x}, {pos.y}
			</p>
		{/if}
		<button on:click={() => core.wait()}>Wait</button>
	{/if}

	{#if !alive}
		<p>You died!</p>
		<button on:click={() => core.restart()}>Restart</button>
	{/if}
</div>

<style>
	div {
		color: white;
		text-align: left;
		margin: 2em;
	}
</style>
