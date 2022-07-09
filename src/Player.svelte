<script>
	import Inventory from './Inventory.svelte';

	export let alive;
	export let hp;
	export let pos;
	export let log;
	export let attack;
	export let inventory;
    export let core;
</script>

<div>
	{#if alive}
		<Inventory {inventory} {core} />

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
	{/if}

	{#if !alive}
		<p>You died!</p>
	{/if}

	{#if log}
		<h2>Logs</h2>
		<pre>{log}</pre>
	{/if}
</div>

<style>
	div {
		color: white;
		text-align: left;
		margin: 2em;
	}
</style>
