<script>
  import { coreOutput } from "@rogueBoi/store.js";

  let alive = $state(false);
  coreOutput.subscribe((c) => (alive = c && c.player != null));
</script>

<nav class="w-6/12 mx-auto my-0">
  <ul class="list-none">
    {#if alive}
      <li class="mx-auto my-4"><a href="#game">Continue</a></li>
    {/if}
    <li class="mx-auto my-4"><a href="#newgame">New Game</a></li>
    <li class="mx-auto my-4"><a href="#options">Options</a></li>
    <li class="mx-auto my-4"><a href="#dungeon-gen">Map generator</a></li>
  </ul>
</nav>
