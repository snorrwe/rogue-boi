<script>
	export let alive;
	export let hp;
	export let pos;
</script>

<div>
	{#if alive}
		{#if hp != null}
			<p id="player-hp">
				Health: {hp.current} / {hp.max}
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
</div>

<style>
	div {
		color: white;
		text-align: left;
		margin: 2em;
	}
</style>
