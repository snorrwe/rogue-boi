import App from './App.svelte';
import wasm from '../core/Cargo.toml';

const init = async () => {
	const core = (await wasm()).initCore();
	core.init(); // initial tick

	new App({
		target: document.body,
		props: {
			core
		}
	});
};

init();
