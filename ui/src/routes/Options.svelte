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
      const binString = atob(str);
      const bytes = Uint8Array.from(binString, (m) => m.codePointAt(0));
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

    const reader = new FileReader();
    reader.onload = function () {
      const data = new Uint8Array(this.result);
      const binStr = Array.from(data, (d) => String.fromCodePoint(d)).join("");
      const base64 = btoa(binStr);

      coreStore.set(null); // ensure save is not overwritten by an existing game
      localStorage.setItem("save", base64);
      window.location.reload(); // force reloading the game state
    };

    reader.readAsArrayBuffer(saveFiles[0]);
  }
</script>

<button class="bg-red-800 my-2 p-2 rounded-2xl" onclick={clearSave}>Clear save game</button>
<button class="bg-green-900 my-2 p-2 rounded-2xl" onclick={downloadSave}
  >Download the current save file</button
>

<form class="my-4 p-2" onsubmit={uploadSave}>
  <h2 class="text-xl">Upload save file</h2>
  <input
    name="save-file"
    class="bg-yellow-600 p-2 rounded-2xl"
    type="file"
    required
    bind:files={saveFiles}
  />
  <input name="submit-save" type="submit" class="bg-green-800 p-2 rounded-2xl" value="Upload" />
</form>
