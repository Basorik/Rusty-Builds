<script lang="ts">
    import {
        commands,
        type DebugStatsResponse,
        type DebugModEntry,
        type DebugComputedStat,
    } from "../../bindings";
    import Header from "../../components/Header.svelte";
    import { getBuildState } from "$lib/buildState.svelte";

    const build = getBuildState();

    let debugData = $state<DebugStatsResponse | null>(null);
    let error = $state<string | null>(null);
    let loading = $state(false);
    let activeTab = $state<"tree" | "class" | "gems" | "items" | "computed">(
        "computed",
    );
    let filterText = $state("");

    async function refresh() {
        loading = true;
        error = null;
        const result = await commands.getDebugStats();
        if (result.status === "ok") {
            debugData = result.data;
        } else {
            error = result.error;
        }
        loading = false;
    }

    // Refresh whenever build-affecting state changes (class, level, bloodline).
    // Reading build.characterClass / level / bloodline here makes $effect track them
    // so the debug view re-loads every time the Header updates Rust state.
    // A short debounce ensures updateBuildInfo() completes before getDebugStats() runs.
    $effect(() => {
        // Track reactive dependencies so this re-runs on any header change.
        const _c = build.characterClass;
        const _l = build.level;
        const _b = build.bloodline;
        const _a = build.ascendancy;
        const timer = setTimeout(() => refresh(), 100);
        return () => clearTimeout(timer);
    });

    let filteredTreeMods = $derived.by(() => {
        if (!debugData) return [];
        const f = filterText.toLowerCase();
        return f
            ? debugData.tree_mods.filter(
                  (m) =>
                      m.stat.toLowerCase().includes(f) ||
                      m.source.toLowerCase().includes(f) ||
                      m.mod_type.toLowerCase().includes(f),
              )
            : debugData.tree_mods;
    });

    let filteredClassMods = $derived.by(() => {
        if (!debugData) return [];
        const f = filterText.toLowerCase();
        return f
            ? debugData.class_mods.filter(
                  (m) =>
                      m.stat.toLowerCase().includes(f) ||
                      m.source.toLowerCase().includes(f) ||
                      m.mod_type.toLowerCase().includes(f),
              )
            : debugData.class_mods;
    });

    let filteredGemMods = $derived.by(() => {
        if (!debugData) return [];
        const f = filterText.toLowerCase();
        return f
            ? debugData.gem_mods.filter(
                  (m) =>
                      m.stat.toLowerCase().includes(f) ||
                      m.source.toLowerCase().includes(f) ||
                      m.mod_type.toLowerCase().includes(f),
              )
            : debugData.gem_mods;
    });

    let filteredItemsMods = $derived.by(() => {
        if (!debugData) return [];
        const f = filterText.toLowerCase();
        return f
            ? debugData.items_mods.filter(
                  (m) =>
                      m.stat.toLowerCase().includes(f) ||
                      m.source.toLowerCase().includes(f) ||
                      m.mod_type.toLowerCase().includes(f),
              )
            : debugData.items_mods;
    });

    let filteredComputed = $derived.by((): [string, DebugComputedStat][] => {
        if (!debugData) return [];
        const entries = (
            Object.entries(debugData.computed) as [
                string,
                DebugComputedStat | undefined,
            ][]
        ).filter((e): e is [string, DebugComputedStat] => e[1] !== undefined);
        entries.sort((a, b) => a[0].localeCompare(b[0]));
        const f = filterText.toLowerCase();
        return f
            ? entries.filter(([name]) => name.toLowerCase().includes(f))
            : entries;
    });
</script>

<main class="debug-page">
    <Header
        bind:characterClass={build.characterClass}
        bind:ascendancy={build.ascendancy}
        bind:bloodline={build.bloodline}
        bind:level={build.level}
    />

    <section class="content">
        <div class="toolbar">
            <h2>ModDB Debug View</h2>
            <button onclick={refresh} disabled={loading}>
                {loading ? "Loading..." : "Refresh"}
            </button>
        </div>

        {#if error}
            <div class="error">{error}</div>
        {/if}

        {#if debugData}
            <div class="stats-summary">
                <span
                    >Tree mods: <strong>{debugData.tree_mods.length}</strong
                    ></span
                >
                <span
                    >Class mods: <strong>{debugData.class_mods.length}</strong
                    ></span
                >
                <span
                    >Gem mods: <strong>{debugData.gem_mods.length}</strong
                    ></span
                >
                <span
                    >Item mods: <strong>{debugData.items_mods.length}</strong
                    ></span
                >
                <span
                    >Computed stats: <strong
                        >{Object.keys(debugData.computed).length}</strong
                    ></span
                >
            </div>

            <div class="tabs">
                <button
                    class:active={activeTab === "computed"}
                    onclick={() => (activeTab = "computed")}
                >
                    Computed
                </button>
                <button
                    class:active={activeTab === "tree"}
                    onclick={() => (activeTab = "tree")}
                >
                    Tree Layer
                </button>
                <button
                    class:active={activeTab === "class"}
                    onclick={() => (activeTab = "class")}
                >
                    Class Layer
                </button>
                <button
                    class:active={activeTab === "gems"}
                    onclick={() => (activeTab = "gems")}
                >
                    Gem Layer
                </button>
                <button
                    class:active={activeTab === "items"}
                    onclick={() => (activeTab = "items")}
                >
                    Items Layer
                </button>
            </div>

            <input
                type="text"
                class="filter"
                placeholder="Filter by stat name, source, or type..."
                bind:value={filterText}
            />

            {#if activeTab === "computed"}
                <table class="debug-table">
                    <thead>
                        <tr>
                            <th>Stat</th>
                            <th class="num">Base</th>
                            <th class="num">Inc%</th>
                            <th class="num">More</th>
                            <th class="num">Total</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each filteredComputed as [name, stat]}
                            <tr>
                                <td class="stat-name">{name}</td>
                                <td class="num">{stat.base}</td>
                                <td class="num"
                                    >{stat.inc ? `${stat.inc}%` : ""}</td
                                >
                                <td class="num"
                                    >{stat.more !== 1
                                        ? `×${stat.more.toFixed(4)}`
                                        : ""}</td
                                >
                                <td class="num total"
                                    >{stat.total % 1 === 0
                                        ? stat.total
                                        : stat.total.toFixed(2)}</td
                                >
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {:else}
                {@const mods =
                    activeTab === "tree"
                        ? filteredTreeMods
                        : activeTab === "class"
                          ? filteredClassMods
                          : activeTab === "gems"
                            ? filteredGemMods
                            : filteredItemsMods}
                <table class="debug-table">
                    <thead>
                        <tr>
                            <th>Stat</th>
                            <th>Type</th>
                            <th class="num">Value</th>
                            <th>Source</th>
                            <th>Flags</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each mods as mod}
                            <tr>
                                <td class="stat-name">{mod.stat}</td>
                                <td>
                                    <span
                                        class="mod-type mod-type-{mod.mod_type.toLowerCase()}"
                                        >{mod.mod_type}</span
                                    >
                                </td>
                                <td class="num">{mod.value}</td>
                                <td class="source">{mod.source}</td>
                                <td class="flags">{mod.flags}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
                {#if mods.length === 0}
                    <p class="empty">
                        No modifiers{filterText ? " matching filter" : ""} in this
                        layer.
                    </p>
                {/if}
            {/if}
        {/if}
    </section>
</main>

<style>
    :global(body) {
        margin: 0;
        background-color: #0e0e10;
        color: #e0d6c2;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    }

    .debug-page {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
    }

    .content {
        max-width: 1200px;
        margin: 0 auto;
        padding: 1.5rem;
        width: 100%;
        box-sizing: border-box;
    }

    .toolbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 1rem;
    }

    .toolbar h2 {
        margin: 0;
        color: #c8b88a;
    }

    .toolbar button {
        padding: 0.4rem 1rem;
        background: #2a2a2e;
        color: #e0d6c2;
        border: 1px solid #555;
        border-radius: 4px;
        cursor: pointer;
    }

    .toolbar button:hover {
        background: #3a3a3e;
    }

    .error {
        color: #ff5555;
        padding: 0.5rem;
        margin-bottom: 1rem;
        background: #2a1010;
        border: 1px solid #ff5555;
        border-radius: 4px;
    }

    .stats-summary {
        display: flex;
        gap: 2rem;
        margin-bottom: 1rem;
        color: #888;
        font-size: 0.9rem;
    }

    .stats-summary strong {
        color: #c8b88a;
    }

    .tabs {
        display: flex;
        gap: 0;
        margin-bottom: 0.75rem;
    }

    .tabs button {
        padding: 0.5rem 1.2rem;
        background: #1a1a1e;
        color: #888;
        border: 1px solid #333;
        cursor: pointer;
        font-size: 0.9rem;
    }

    .tabs button:first-child {
        border-radius: 4px 0 0 4px;
    }

    .tabs button:last-child {
        border-radius: 0 4px 4px 0;
    }

    .tabs button.active {
        background: #2a2a2e;
        color: #e0d6c2;
        border-color: #555;
    }

    .filter {
        width: 100%;
        padding: 0.5rem;
        margin-bottom: 0.75rem;
        background: #1a1a1e;
        color: #e0d6c2;
        border: 1px solid #333;
        border-radius: 4px;
        box-sizing: border-box;
        font-size: 0.9rem;
    }

    .filter:focus {
        outline: none;
        border-color: #c8b88a;
    }

    .debug-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.85rem;
    }

    .debug-table th {
        text-align: left;
        padding: 0.5rem 0.75rem;
        border-bottom: 2px solid #333;
        color: #c8b88a;
        position: sticky;
        top: 0;
        background: #0e0e10;
    }

    .debug-table td {
        padding: 0.35rem 0.75rem;
        border-bottom: 1px solid #1e1e22;
    }

    .debug-table tbody tr:hover {
        background: #1a1a1e;
    }

    .num {
        text-align: right;
        font-variant-numeric: tabular-nums;
    }

    .stat-name {
        color: #88bbee;
        font-family: monospace;
    }

    .total {
        color: #c8b88a;
        font-weight: 600;
    }

    .source {
        color: #888;
        font-size: 0.8rem;
    }

    .flags {
        color: #666;
        font-size: 0.8rem;
        font-family: monospace;
    }

    .mod-type {
        display: inline-block;
        padding: 0.1rem 0.4rem;
        border-radius: 3px;
        font-size: 0.75rem;
        font-weight: 600;
    }

    .mod-type-base {
        background: #1a2a1a;
        color: #66cc66;
    }

    .mod-type-inc {
        background: #1a1a2a;
        color: #6688ee;
    }

    .mod-type-more {
        background: #2a1a2a;
        color: #cc66cc;
    }

    .mod-type-flag {
        background: #2a2a1a;
        color: #cccc66;
    }

    .mod-type-override {
        background: #2a1a1a;
        color: #cc6666;
    }

    .empty {
        text-align: center;
        color: #555;
        padding: 2rem;
    }
</style>
