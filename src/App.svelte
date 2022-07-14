<script>
	import { canvasStore, svgStore } from './store.js';
	import { writable } from 'svelte/store';
	import { onMount } from 'svelte';
	import Grid from './Grid.svelte';
	import Highlight from './Highlight.svelte';
	import Log from './Log.svelte';
	import Player from './Player.svelte';

	export let core;

	const coreOutput = writable(core.getOutput());
	let selected = writable(null);
	coreOutput.subscribe((c) => selected.set(c && c.selected && core.fetchEntity(c.selected)));
	const inventory = writable(core.getInventory());
	let last = new Date().getTime();

	canvasStore.subscribe((canvas) => {
		console.log(canvas);
		core.setCanvas(canvas);
	});

	document.addEventListener('keydown', (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	});

	const gameLoop = () => {
		const now = new Date().getTime();

		core.tick(now - last);

		coreOutput.set(core.getOutput());
		inventory.set(core.getInventory());
		last = now;
		if ($selected && $selected.id) {
			selected.set(core.fetchEntity($selected.id));
		}

		requestAnimationFrame(gameLoop);
	};
	requestAnimationFrame(gameLoop);

	onMount(() => {
		const icons = core.icons();
		icons.forEach((key) =>
			fetch(`icons/${key}.svg`)
				.then((resp) => resp.text())
				.then((pl) => {
					svgStore.update((s) => ({ ...s, [key]: pl }));
					const parser = new DOMParser();
					const doc = parser.parseFromString(pl, 'application/xml');
					const paths = doc.querySelectorAll('path');
					// TODO: multiple paths?
					const inner = paths[0].attributes.d;
					core.setIconPayload(key, inner.nodeValue);
				})
		);
	});
</script>

<main>
	<div class="content">
		<div>
			{#if core != null && $coreOutput != null}
				<Grid coreOutput={$coreOutput} {core} />
			{/if}
		</div>
		<div>
			{#if $selected != null}
				<Highlight selected={$selected} {core} />
			{/if}
			<div>
				<Player
					inventory={$inventory}
					alive={$coreOutput.player != null}
					hp={$coreOutput.player && $coreOutput.player.playerHp}
					pos={$coreOutput.player && $coreOutput.player.playerPosition}
					attack={$coreOutput.player && $coreOutput.player.playerAttack}
					{core}
				/>
			</div>
		</div>
		<div class="log">
			<h2>Logs</h2>
			<Log log={$coreOutput.log} />
		</div>
	</div>
</main>

<style>
	main {
		padding: 2em;
		margin: 0 auto;
		color: white;
		min-width: 0px;
	}

	.content {
		display: grid;
		grid-template-columns: minmax(0, 2fr) repeat(2, 1fr);
	}

	.log {
		max-height: 100%;
		overflow: auto;
	}
</style>
