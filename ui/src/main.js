import App from "./App.svelte";
import wasm, { initCore } from "rogue-boi-core";

const init = async () => {
  await wasm();
  const core = initCore();

  new App({
    target: document.body,
    props: {
      core
    }
  });
};

init();
