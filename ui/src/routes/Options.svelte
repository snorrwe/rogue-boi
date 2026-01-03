<script>
  import { coreStore } from "@rogueBoi/store.js";

  function clearSave() {
    localStorage.removeItem("save");
    coreStore.set(null);
    window.location.reload();
  }

  function downloadSave() {
    const str = localStorage.getItem("save");
    let url = "data:text,null";
    if (str) {
      let binary = Buffer.from(str, "base64");
      let bytes = new Uint8Array(binary.length);

      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }

      const blob = new Blob([bytes], { type: "application/octet-stream" });
      url = URL.createObjectURL(blob);
    }
    let link = document.createElement("a");
    link.href = url;
    link.download = "rogueBoi.save";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  /** @type{FileList} */
  let saveFiles;

  function uploadSave(e) {
    e.preventDefault();
    saveFiles[0].bytes().then((data) => {
      coreStore.set(null); // ensure save is not overwritten by an existing game
      localStorage.setItem("save", data.toString("base64"));
      window.location.reload(); // force reloading the game state
    });
  }
</script>

<button class="bg-red-800 my-2 p-2 rounded-2xl" onclick={clearSave}>Clear save game</button>
<button class="bg-green-900 my-2 p-2 rounded-2xl" onclick={downloadSave}
  >Download the current save file</button
>

<form class="my-4 p-2" onsubmit={uploadSave}>
  <h2 class="text-xl">Upload save file</h2>
  <input class="bg-yellow-600 p-2 rounded-2xl" type="file" required bind:files={saveFiles} />
  <input type="submit" class="bg-green-800 p-2 rounded-2xl" value="Upload" />
</form>
