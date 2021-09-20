import App from './App.svelte';
import wasm from '../core/Cargo.toml';

const init = async () => {
	const core = await wasm();

	core.init_core();

	const app = new App({
		target: document.body,
		props: {
			// https://svelte.dev/docs#Creating_a_component
			greet: core.greet()
		}
	});
};

init();
