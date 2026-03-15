<script lang="ts">
    let { selectedCount = 0, ascSelectedCount = 0 }: { selectedCount?: number; ascSelectedCount?: number } = $props();
    import { goto } from "$app/navigation";

    function Menu() {
        goto("/");
    }
</script>

<aside class="sidebar">
    <section class="actions">
        <button class="btn-primary" onclick={Menu}> Menu </button>
    </section>

    <h2 class="sidebar-title">Build Info</h2>

    <div class="stat-section">
        <div class="stat-label">Selected Nodes</div>
        <div class="stat-value">{selectedCount}</div>
    </div>

    <div class="stat-section">
        <div class="stat-label">Ascendency Points</div>
        <div class="stat-value asc-value">{ascSelectedCount} / 8</div>
    </div>

    <hr class="divider" />

    <div class="placeholder">
        <span class="placeholder-text">More stats coming soon…</span>
    </div>
</aside>

<style>
    .sidebar {
        width: 240px;
        min-width: 240px;
        height: 100vh;
        background: #111;
        border-right: 1px solid #2a2a2a;
        display: flex;
        flex-direction: column;
        padding: 16px;
        box-sizing: border-box;
        font-family: sans-serif;
        color: #dfcf99;
        overflow-y: auto;
    }

    .sidebar-title {
        margin: 0 0 20px 0;
        font-size: 1.1em;
        color: #fff;
        letter-spacing: 0.05em;
        text-transform: uppercase;
    }

    .stat-section {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 0;
    }

    .stat-label {
        font-size: 0.9em;
        color: #aaa;
    }

    .stat-value {
        font-size: 1.4em;
        font-weight: bold;
        color: #4488ff;
    }

    .stat-value.asc-value {
        color: #c8a95e;
    }

    .divider {
        border: none;
        border-top: 1px solid #2a2a2a;
        margin: 16px 0;
    }

    .placeholder {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .placeholder-text {
        color: #555;
        font-size: 0.85em;
        font-style: italic;
    }

    .btn-primary {
        padding: 0.7em 2em;
        font-size: 1.05rem;
        font-weight: 600;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        color: #0e0e10;
        background: linear-gradient(135deg, #c8a95e, #a38638);
        transition:
            transform 0.15s,
            box-shadow 0.15s;
        box-shadow: 0 2px 8px rgba(200, 169, 94, 0.25);
    }

    .btn-primary:hover {
        transform: translateY(-1px);
        box-shadow: 0 4px 14px rgba(200, 169, 94, 0.4);
    }

    .btn-primary:active {
        transform: translateY(0);
    }
</style>
