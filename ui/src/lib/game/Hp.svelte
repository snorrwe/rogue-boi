<script>
  export let currentHp;
  export let maxHp;

  let t;
  $: {
    t = currentHp / maxHp;
    // clamp to 0,1
    // for some reason the css animation is messed up at t=1
    // bad lerp implementation?
    t = Math.max(Math.min(t, 0.999999), 0.0);
  }
</script>

<div class="hp-bar" style="--t:{t * 100};"><p>{currentHp} / {maxHp}</p></div>

<style>
  @keyframes hp {
    0% {
      --bg: red;
      --txt: white;
    }
    60% {
      --bg: orange;
      --txt: black;
    }
    100% {
      --bg: lime;
      --txt: black;
    }
  }

  .hp-bar {
    width: 200px;
    height: 20px;
    animation: 100s linear calc(-1s * var(--t)) paused hp;
    background-color: var(--bg);
    color: var(--txt);
  }
</style>
