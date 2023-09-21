import App from "./App.svelte";
import wasm from "../rogue-boi-core/Cargo.toml";

const init = async () => {
  const core = (
    await wasm({
      importHook: (path) => {
        // force relative url
        if (path.startsWith("/")) {
          return path.slice(1);
        }
        return path;
      }
    })
  ).initCore();

  new App({
    target: document.body,
    props: {
      core
    }
  });
};

init();
