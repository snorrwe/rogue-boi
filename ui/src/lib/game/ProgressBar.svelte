<script>
  let { current, max, minColorBg, minColorFg, maxColorBg, maxColorFg, midColorBg, midColorFg } =
    $props();

  let t = $state();
  $effect(() => {
    let tt = current / max;
    // clamp to 0,1
    // for some reason the css animation is messed up at t=1
    // bad lerp implementation?
    t = Math.max(Math.min(tt, 0.999999), 0.0);
  });
</script>

<div
  class="bar"
  style="--t:{t *
    100}; --minColorBg:{minColorBg}; --minColorFg:{minColorFg}; --maxColorBg:{maxColorBg}; --maxColorFg:{maxColorFg}; --midColorBg:{midColorBg}; --midColorFg:{midColorFg}; "
>
  <p>{current} / {max}</p>
</div>

<style>
  @keyframes progress {
    0% {
      --bg: var(--minColorBg);
      --txt: var(--minColorFg);
    }
    60% {
      --bg: var(--midColorBg);
      --txt: var(--midColorFg);
    }
    100% {
      --bg: var(--maxColorBg);
      --txt: var(--maxColorFg);
    }
  }

  .bar {
    width: 200px;
    height: 20px;
    animation: 100s linear calc(-1s * var(--t)) paused progress;
    background-color: var(--bg);
    color: var(--txt);
  }
</style>
