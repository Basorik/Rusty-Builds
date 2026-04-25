<script lang="ts">
    import type { BuildStats } from "../bindings";
    import { commands } from "../bindings";
    import { getBuildState } from "$lib/buildState.svelte";

    const build = getBuildState();

    let {
        selectedCount = 0,
        ascSelectedCount = 0,
        buildStats = null,
    }: {
        selectedCount?: number;
        ascSelectedCount?: number;
        buildStats?: BuildStats | null;
    } = $props();
    import { goto } from "$app/navigation";

    function Menu() {
        goto("/");
    }

    /** Flat list of all non-support gems across all skill groups for the dropdown */
    let allActiveGems = $derived(
        build.skillGroups.flatMap((group) =>
            group.gems
                .map((gem, idx) => ({ group, gem, idx }))
                .filter(({ gem }) => !gem.is_support),
        ),
    );

    /** The select value encoding the currently active GemRef */
    let activeGemSelectValue = $derived(
        build.activeGem
            ? `${build.activeGem.group_id}-${build.activeGem.gem_index}`
            : "",
    );

    async function onActiveGemChange(e: Event) {
        const val = (e.target as HTMLSelectElement).value;
        const newRef = val
            ? (() => {
                  const [gid, gi] = val.split("-");
                  return { group_id: Number(gid), gem_index: Number(gi) };
              })()
            : null;
        const result = await commands.setActiveGem(newRef);
        if (result.status === "ok") {
            build.activeGem = newRef;
            build.buildStats = result.data;
        }
    }

    function fmt(n: number, decimals = 0): string {
        return n.toLocaleString("en-US", {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals,
        });
    }

    function fmtPct(n: number, decimals = 1): string {
        return `${n.toFixed(decimals)}%`;
    }

    function resistClass(r: number): string {
        if (r < 0) return "resist-neg";
        if (r >= 75) return "resist-cap";
        return "resist-ok";
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

    {#if allActiveGems.length > 0}
        <hr class="divider" />
        <h3 class="stat-group-title">Active Skill</h3>
        <select
            class="active-gem-select"
            value={activeGemSelectValue}
            onchange={onActiveGemChange}
        >
            <option value="">— None —</option>
            {#each allActiveGems as { group, gem, idx }}
                <option value="{group.id}-{idx}">{gem.name}</option>
            {/each}
        </select>
    {/if}

    <hr class="divider" />

    {#if buildStats}
        <!-- Offence summary (only when active gem exists) -->
        {#if buildStats.offence}
            {@const o = buildStats.offence}
            <h3 class="stat-group-title">Offence</h3>
            <div class="stat-section">
                <div class="stat-label gold-label">Total DPS</div>
                <div class="stat-value gold-value">{fmt(o.total_dps, 1)}</div>
            </div>
            <div class="stat-section">
                <div class="stat-label">Hit DPS</div>
                <div class="stat-value">{fmt(o.hit_dps, 1)}</div>
            </div>
            <div class="stat-section">
                <div class="stat-label">Avg Hit</div>
                <div class="stat-value">{fmt(o.average_hit, 1)}</div>
            </div>
            <div class="stat-section">
                <div class="stat-label">Crit</div>
                <div class="stat-value">
                    {fmtPct(o.crit_chance)} / {fmt(o.crit_multiplier * 100, 0)}%
                </div>
            </div>
            <div class="stat-section">
                <div class="stat-label">{o.is_attack ? "APS" : "Casts/s"}</div>
                <div class="stat-value">{o.speed.toFixed(2)}</div>
            </div>
            {#if o.is_attack}
                <div class="stat-section">
                    <div class="stat-label">Hit Chance</div>
                    <div class="stat-value">{fmtPct(o.hit_chance)}</div>
                </div>
            {/if}
            {#if o.dot_dps > 0}
                <div class="stat-section">
                    <div class="stat-label">DoT DPS</div>
                    <div class="stat-value">{fmt(o.dot_dps, 1)}</div>
                </div>
            {/if}
        {/if}

        <h3 class="stat-group-title" style="margin-top: 8px">Resources</h3>
        <div class="stat-section">
            <div class="stat-label life-label">Life</div>
            <div class="stat-value life-value">{buildStats.defence.life}</div>
        </div>
        <div class="stat-section">
            <div class="stat-label mana-label">Mana</div>
            <div class="stat-value mana-value">{buildStats.defence.mana}</div>
        </div>
        {#if buildStats.defence.energy_shield > 0}
            <div class="stat-section">
                <div class="stat-label es-label">Energy Shield</div>
                <div class="stat-value es-value">
                    {buildStats.defence.energy_shield}
                </div>
            </div>
        {/if}

        <h3 class="stat-group-title" style="margin-top: 8px">Defence</h3>
        {#if buildStats.defence.armour > 0}
            <div class="stat-section">
                <div class="stat-label">Armour</div>
                <div class="stat-value">{fmt(buildStats.defence.armour)}</div>
            </div>
        {/if}
        {#if buildStats.defence.evasion > 0}
            <div class="stat-section">
                <div class="stat-label">Evasion</div>
                <div class="stat-value">{fmt(buildStats.defence.evasion)}</div>
            </div>
        {/if}
        <div class="resist-row">
            <div class="resist-cell">
                <div class="resist-label fire-res">Fire</div>
                <div
                    class="resist-val {resistClass(
                        buildStats.defence.fire_resist,
                    )}"
                >
                    {buildStats.defence.fire_resist}%
                </div>
            </div>
            <div class="resist-cell">
                <div class="resist-label cold-res">Cold</div>
                <div
                    class="resist-val {resistClass(
                        buildStats.defence.cold_resist,
                    )}"
                >
                    {buildStats.defence.cold_resist}%
                </div>
            </div>
            <div class="resist-cell">
                <div class="resist-label light-res">Light</div>
                <div
                    class="resist-val {resistClass(
                        buildStats.defence.lightning_resist,
                    )}"
                >
                    {buildStats.defence.lightning_resist}%
                </div>
            </div>
            <div class="resist-cell">
                <div class="resist-label chaos-res">Chaos</div>
                <div
                    class="resist-val {resistClass(
                        buildStats.defence.chaos_resist,
                    )}"
                >
                    {buildStats.defence.chaos_resist}%
                </div>
            </div>
        </div>

        <h3 class="stat-group-title" style="margin-top: 8px">Attributes</h3>
        <div class="stat-section">
            <div class="stat-label str-label">Strength</div>
            <div class="stat-value str-value">{buildStats.total_strength}</div>
        </div>
        <div class="stat-section">
            <div class="stat-label dex-label">Dexterity</div>
            <div class="stat-value dex-value">{buildStats.total_dexterity}</div>
        </div>
        <div class="stat-section">
            <div class="stat-label int-label">Intelligence</div>
            <div class="stat-value int-value">
                {buildStats.total_intelligence}
            </div>
        </div>
    {:else}
        <div class="placeholder">
            <span class="placeholder-text">Select nodes to see stats…</span>
        </div>
    {/if}

    {#if build.lastIpcMs > 0}
        <hr class="divider" />
        <h3 class="stat-group-title perf-title">Performance</h3>
        {@const rustMs = buildStats ? buildStats.calc_time_us / 1000 : 0}
        {@const overheadMs = build.lastIpcMs - rustMs}
        <div class="stat-section">
            <div class="stat-label perf-label">Rust calc</div>
            <div class="stat-value perf-value">{rustMs.toFixed(2)} ms</div>
        </div>
        <div class="stat-section">
            <div class="stat-label perf-label">IPC round-trip</div>
            <div class="stat-value perf-value">
                {build.lastIpcMs.toFixed(2)} ms
            </div>
        </div>
        <div class="stat-section">
            <div class="stat-label perf-label">IPC overhead</div>
            <div class="stat-value perf-overhead">
                {overheadMs.toFixed(2)} ms
            </div>
        </div>
    {/if}
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
        padding: 4px 0;
    }

    .stat-label {
        font-size: 0.85em;
        color: #aaa;
    }

    .stat-value {
        font-size: 1.1em;
        font-weight: bold;
        color: #4488ff;
    }

    .stat-value.asc-value {
        color: #c8a95e;
    }
    .gold-label {
        color: #c8a95e;
    }
    .gold-value {
        color: #c8a95e;
        font-size: 1.2em;
    }

    .divider {
        border: none;
        border-top: 1px solid #2a2a2a;
        margin: 12px 0;
    }

    .stat-group-title {
        margin: 4px 0 6px 0;
        font-size: 0.72em;
        color: #666;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        font-weight: 600;
    }

    /* Resources */
    .life-label {
        color: #e05030;
    }
    .mana-label {
        color: #5b9bd5;
    }
    .es-label {
        color: #9ecfde;
    }
    .life-value {
        color: #e05030;
    }
    .mana-value {
        color: #5b9bd5;
    }
    .es-value {
        color: #9ecfde;
    }

    /* Attributes */
    .str-label {
        color: #c87448;
    }
    .dex-label {
        color: #5cb85c;
    }
    .int-label {
        color: #5b9bd5;
    }
    .str-value {
        color: #c87448;
    }
    .dex-value {
        color: #5cb85c;
    }
    .int-value {
        color: #5b9bd5;
    }

    /* Resist grid */
    .resist-row {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr;
        gap: 2px;
        margin: 4px 0;
    }
    .resist-cell {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 3px 0;
    }
    .resist-label {
        font-size: 0.68em;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        opacity: 0.75;
    }
    .resist-val {
        font-size: 0.92em;
        font-weight: bold;
    }
    .fire-res {
        color: #e04428;
    }
    .cold-res {
        color: #5bb8d4;
    }
    .light-res {
        color: #d4b83c;
    }
    .chaos-res {
        color: #d44be0;
    }
    .resist-neg {
        color: #e05050;
    }
    .resist-ok {
        color: #d4c080;
    }
    .resist-cap {
        color: #5cb85c;
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

    .perf-title {
        color: #555;
    }
    .perf-label {
        color: #555;
        font-size: 0.78em;
    }
    .perf-value {
        color: #555;
        font-size: 0.9em;
        font-weight: normal;
        font-variant-numeric: tabular-nums;
    }
    .perf-overhead {
        color: #664444;
        font-size: 0.9em;
        font-weight: normal;
        font-variant-numeric: tabular-nums;
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 16px;
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

    .btn-nav {
        padding: 0.45em 1.2em;
        font-size: 0.85rem;
        font-weight: 500;
        border: 1px solid #3a3730;
        border-radius: 6px;
        cursor: pointer;
        color: #e0d6c2;
        background: #1a1a1e;
        width: 100%;
        transition:
            border-color 0.15s,
            background 0.15s;
    }

    .btn-nav:hover {
        border-color: #c8a96e;
        background: #22201a;
    }

    .active-gem-select {
        width: 100%;
        padding: 0.4em 0.5em;
        background: #1a1a1e;
        border: 1px solid #3a3730;
        border-radius: 4px;
        color: #e0d6c2;
        font-size: 0.85rem;
        cursor: pointer;
        margin-top: 4px;
    }

    .active-gem-select:focus {
        outline: none;
        border-color: #c8a95e;
    }
</style>
