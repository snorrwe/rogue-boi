import App from "./App.svelte";
import wasm, { initCore } from "rogue-boi-core";
import { mount } from "svelte";

const init = async () => {
  await wasm();
  const core = initCore();

  mount(App, {
    target: document.body,
    props: {
      core
    }
  });
};

init();
