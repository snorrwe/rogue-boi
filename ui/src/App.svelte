<script>
  import "./app.css";
  import { onMount } from "svelte";
  import { fetchIcon, coreStore, coreOutput, selected } from "@rogueBoi/store.js";
  import Menu from "./routes/Menu.svelte";
  import Game from "./routes/Game.svelte";
  import Options from "./routes/Options.svelte";
  import DungeonGenerator from "./routes/DungeonGenerator.svelte";

  let { core } = $props();
  let page = $state(Menu);

  function saveGame() {
    if ($coreStore) {
      let pl = $coreStore.save();

      localStorage.setItem("save", pl);
    }
  }

  onMount(() => {
    core.icons().forEach((icon) => {
      fetchIcon({ src: `icons/${icon}.svg`, name: icon });
    });

    const saveGame = localStorage.getItem("save");

    if (saveGame != null) {
      try {
        console.log("Loading previous save");
        core.load(saveGame);
        coreOutput.set(core.getOutput());
      } catch (err) {
        console.error("Failed to load save game", err);
        localStorage.removeItem("save");
      }
    }

    coreStore.set(core);
    routeChange();
  });

  document.addEventListener("keydown", (event) => {
    core.pushEvent({ ty: "KeyDown", key: event.key });
  });

  const routeChange = () => {
    let factory =
      {
        "#game": () => Game,
        "#newgame": () => {
          selected.set(null);

          if (core) {
            core.restart();
            window.location.hash = "game";
            return Game;
          }

          return Menu;
        },
        "#menu": () => Menu,
        "#options": () => Options,
        "#dungeon-gen": () => DungeonGenerator
      }[location.hash] || (() => Menu);

    page = factory();
  };

  const SvelteComponent = $derived(page);
</script>

<svelte:window onunload={saveGame} onhashchange={routeChange} />

<main class="p-2 my-0 mx-auto text-white min-w-0">
  <header>
    <a href="#menu">Back to menu</a>
  </header>
  <SvelteComponent />
</main>
