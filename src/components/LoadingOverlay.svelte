<script lang="ts">
    let {
        step = "Initializing…",
        fraction = 0,
    }: { step: string; fraction: number } = $props();

    let pct = $derived(Math.round(fraction * 100));
</script>

<div class="overlay">
    <div class="panel">
        <h1 class="title">Rusty Builds</h1>
        <p class="subtitle">Path of Exile Build Planner</p>

        <div class="bar-track">
            <div class="bar-fill" style="width: {pct}%"></div>
        </div>

        <p class="step">{step}</p>
        <p class="pct">{pct}%</p>
    </div>
</div>

<style>
    .overlay {
        position: fixed;
        inset: 0;
        background: #0d0d0d;
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 9999;
    }

    .panel {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        width: 420px;
    }

    .title {
        font-family: "Fontin SmallCaps", "Palatino Linotype", serif;
        font-size: 2.4rem;
        color: #af6025;
        margin: 0;
        letter-spacing: 2px;
        text-shadow: 0 0 12px #7a3c0880;
    }

    .subtitle {
        font-size: 0.85rem;
        color: #7d7065;
        margin: 0;
        letter-spacing: 1px;
        text-transform: uppercase;
    }

    .bar-track {
        width: 100%;
        height: 8px;
        background: #1e1811;
        border: 1px solid #3a2a18;
        border-radius: 4px;
        overflow: hidden;
        margin-top: 8px;
    }

    .bar-fill {
        height: 100%;
        background: linear-gradient(90deg, #7a3c08, #c8722a);
        border-radius: 4px;
        transition: width 0.15s ease-out;
    }

    .step {
        font-size: 0.8rem;
        color: #a09080;
        margin: 0;
        min-height: 1em;
    }

    .pct {
        font-size: 0.75rem;
        color: #5a4a3a;
        margin: 0;
    }
</style>
