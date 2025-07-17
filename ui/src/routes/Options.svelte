<script>
  import { coreStore } from "@rogueBoi/store.js";

  function clearSave() {
    localStorage.removeItem("save");
    coreStore.set(null);
    window.location.reload();
  }
</script>

<button class="bg-red-800 my-2" onclick={clearSave}>Clear save game</button>
