import App from './App.svelte';
import wasm from '../core/Cargo.toml';

const init = async () => {
	const core = (await wasm()).initCore();
    core.tick(0); // initial tick

	const app = new App({
		target: document.body,
		props: {
			core
		}
	});
};

init();
