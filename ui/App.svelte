<script>
	import { onMount } from 'svelte';
	import { fetchIcon, coreStore, coreOutputStore } from '@rogueBoi/store.js';
	import Menu from './routes/Menu.svelte';
	import Game from './routes/Game.svelte';

	export let core;
	let page = Menu;

	onMount(() => {
		core.icons().forEach((icon) => {
			fetchIcon({ src: `icons/${icon}.svg`, name: icon });
		});
		const saveGame = localStorage.getItem('save');
		if (saveGame != null) {
			try {
				console.log('Loading previous save');
				core.load(saveGame);
				coreOutputStore.set(core.getOutput());
			} catch (err) {
				console.error('Failed to load save game', err);
				localStorage.removeItem('save');
			}
		}
		coreStore.set(core);
		routeChange();
	});

	function saveGame() {
		if (core) {
			let pl = core.save();
			console.log(pl.length);
			localStorage.setItem('save', pl);
		}
	}

	document.addEventListener('keydown', (event) => {
		core.pushEvent({
			ty: 'KeyDown',
			key: event.key
		});
	});

	const routeChange = () => {
		let factory =
			{
				'#game': () => Game,
				'#newgame': () => {
					if (core) {
						core.restart();
						window.location.hash = 'game';
						return Game;
					}
					return Menu;
				},
				'#menu': () => Menu
			}[location.hash] || (() => Menu);
		page = factory();
	};
</script>

<svelte:window on:unload={saveGame} on:hashchange={routeChange} />

<main>
	<header>
		<a href="#menu">Back to menu</a>
	</header>
	<svelte:component this={page} />
</main>

<style>
	main {
		padding: 2em;
		margin: 0 auto;
		color: white;
		min-width: 0px;
	}
</style>
