<script>
	export let log;
</script>

{#if log}
	<ul>
		{#each log as line}
			<li>{@html line}</li>
		{/each}
	</ul>
{/if}

<style>
	ul {
		list-style: none;
	}
</style>
