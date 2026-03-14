<script>
  import { coreOutput } from "@rogueBoi/store.js";

  let shop = $derived($coreOutput?.shop);
  let inventory = $derived(shop?.inventory);
</script>

<h1 class="text-3xl">Shop</h1>

<pre>
====inventory====
    {JSON.stringify(inventory, null, 4)}
</pre>
