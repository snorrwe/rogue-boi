<script>
  import { coreStore } from "@rogueBoi/store.js";

  function clearSave() {
    localStorage.removeItem("save");
    coreStore.set(null);
    window.location.reload();
  }

  function downloadSave() {
    const data = localStorage.getItem("save");

    let link = document.createElement("a");
    link.href = `data:text,${data}`;
    link.download = "rogueBoi.save";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
</script>

<button class="bg-red-800 my-2 p-2 rounded-2xl" onclick={clearSave}>Clear save game</button>
<button class="bg-amber-900 my-2 p-2 rounded-2xl" onclick={downloadSave}
  >Download the current save file</button
>
