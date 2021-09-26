<script>
	export let core;

    import Grid from "./Grid.svelte"
</script>

<main>
    {#if core != null}
        <Grid grid={core.tick()} core={core} />
    {/if}
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
        background: black;
	}

	pre {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
