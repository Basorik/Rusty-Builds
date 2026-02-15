<script lang="ts">
    import { goto } from "$app/navigation";

    // Placeholder saved builds — will be replaced with actual load logic later
    interface SavedBuild {
        id: string;
        name: string;
        class: string;
        lastModified: string;
        nodeCount: number;
    }

    let savedBuilds = $state<SavedBuild[]>([
        // Placeholder data — uncomment to preview the list UI:
        // { id: "1", name: "Lightning Arc Witch", class: "Witch", lastModified: "2026-02-10", nodeCount: 97 },
        // { id: "2", name: "Cyclone Slayer", class: "Marauder", lastModified: "2026-02-08", nodeCount: 112 },
    ]);

    function newBuild() {
        goto("/skilltree");
    }

    function loadBuild(id: string) {
        // TODO: load build data from storage, then navigate
        goto("/skilltree");
    }

    function deleteBuild(id: string) {
        // TODO: delete from storage
        savedBuilds = savedBuilds.filter((b) => b.id !== id);
    }
</script>

<main class="home">
    <header class="hero">
        <h1>Rusty Builds</h1>
        <p class="subtitle">Path of Exile Skill Tree Planner</p>
    </header>

    <section class="actions">
        <button class="btn-primary" onclick={newBuild}> + New Build </button>
    </section>

    <section class="builds-section">
        <h2>Saved Builds</h2>

        {#if savedBuilds.length === 0}
            <div class="empty-state">
                <p>No saved builds yet.</p>
                <p class="hint">Create a new build to get started!</p>
            </div>
        {:else}
            <ul class="build-list">
                {#each savedBuilds as build (build.id)}
                    <li class="build-card">
                        <button
                            class="build-card-body"
                            onclick={() => loadBuild(build.id)}
                        >
                            <div class="build-info">
                                <span class="build-name">{build.name}</span>
                                <span class="build-meta"
                                    >{build.class} &middot; {build.nodeCount} nodes</span
                                >
                            </div>
                            <span class="build-date">{build.lastModified}</span>
                        </button>
                        <button
                            class="btn-delete"
                            onclick={() => deleteBuild(build.id)}
                            title="Delete build"
                        >
                            ✕
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </section>
</main>

<style>
    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        font-weight: 400;
        color: #e0d6c2;
        background-color: #0e0e10;
        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }

    .home {
        max-width: 640px;
        margin: 0 auto;
        padding: 6vh 1.5rem 4rem;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    /* ---- Hero ---- */
    .hero {
        text-align: center;
    }

    h1 {
        margin: 0;
        font-size: 2.4rem;
        color: #c8a95e;
        letter-spacing: 0.04em;
    }

    .subtitle {
        margin: 0.3rem 0 0;
        font-size: 0.95rem;
        color: #8a8578;
    }

    /* ---- Actions ---- */
    .actions {
        display: flex;
        justify-content: center;
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

    /* ---- Builds list ---- */
    .builds-section {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    h2 {
        margin: 0;
        font-size: 1.15rem;
        color: #a89c84;
        border-bottom: 1px solid #2a2723;
        padding-bottom: 0.4rem;
    }

    .empty-state {
        text-align: center;
        padding: 2rem 0;
        color: #5a5448;
    }

    .empty-state p {
        margin: 0.25rem 0;
    }

    .hint {
        font-size: 0.85rem;
    }

    .build-list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .build-card {
        display: flex;
        align-items: center;
        background: #1a1918;
        border: 1px solid #2a2723;
        border-radius: 6px;
        overflow: hidden;
        transition: border-color 0.2s;
    }

    .build-card:hover {
        border-color: #c8a95e55;
    }

    .build-card-body {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem 1rem;
        background: none;
        border: none;
        color: inherit;
        font: inherit;
        cursor: pointer;
        text-align: left;
    }

    .build-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .build-name {
        font-weight: 600;
        color: #e0d6c2;
    }

    .build-meta {
        font-size: 0.8rem;
        color: #7a7264;
    }

    .build-date {
        font-size: 0.8rem;
        color: #5a5448;
        white-space: nowrap;
    }

    .btn-delete {
        padding: 0.6rem 0.8rem;
        background: none;
        border: none;
        color: #5a5448;
        font-size: 1rem;
        cursor: pointer;
        transition: color 0.15s;
    }

    .btn-delete:hover {
        color: #e05050;
    }
</style>
