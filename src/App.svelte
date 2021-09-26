<script>
    import { writable } from 'svelte/store';
    import Grid from "./Grid.svelte"

	export let core;

    const grid = writable(null);
	let last = new Date().getTime();

    document.addEventListener('keydown', (event) => {
        core.pushEvent({
            ty: 'KeyDown',
            key: event.key
        });
    });

	const gameLoop = () => {
		const now = new Date().getTime();

		core.tick(now - last);

        grid.set(core.getGrid());
		last = now;

		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);
</script>

<main>
    {#if core != null && $grid != null}
        <Grid grid={$grid} core={core} />
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
