<script lang="ts">
    import { goto } from "$app/navigation";
    import { getBuildState } from "$lib/buildState.svelte";
    import type { DefenceResult, OffenceResult } from "../../bindings";

    const build = getBuildState();

    // Active tab within the calcs page
    let activeTab = $state<"offence" | "defence">("offence");

    let d = $derived(build.buildStats?.defence ?? null);
    let o = $derived(build.buildStats?.offence ?? null);

    function fmt(n: number, dec = 0): string {
        return n.toLocaleString("en-US", {
            minimumFractionDigits: dec,
            maximumFractionDigits: dec,
        });
    }

    function fmtPct(n: number, dec = 1): string {
        return `${n.toFixed(dec)}%`;
    }

    function resistClass(r: number): string {
        if (r < 0) return "neg";
        if (r >= 75) return "cap";
        return "ok";
    }
</script>

<div class="calcs-page">
    <!-- Left nav panel -->
    <aside class="nav-panel">
        <button class="btn-back" onclick={() => goto("/skilltree")}
            >← Tree</button
        >
        <h2 class="panel-title">Calculations</h2>

        <nav class="tab-list">
            <button
                class="tab-btn {activeTab === 'offence' ? 'active' : ''}"
                onclick={() => (activeTab = "offence")}
            >
                Offence
            </button>
            <button
                class="tab-btn {activeTab === 'defence' ? 'active' : ''}"
                onclick={() => (activeTab = "defence")}
            >
                Defence
            </button>
        </nav>
    </aside>

    <!-- Main content -->
    <main class="content">
        {#if !build.buildStats}
            <div class="empty-state">
                <p>No build stats available yet.</p>
                <p>Select nodes on the skill tree to begin.</p>
            </div>
        {:else if activeTab === "offence"}
            <div class="section-header">
                <h1 class="section-title">Offence</h1>
                {#if !o}
                    <p class="hint">
                        Select an active gem in the skill tree sidebar to see
                        offence results.
                    </p>
                {/if}
            </div>

            {#if o}
                <!-- Headline DPS -->
                <div class="card">
                    <h3 class="card-title">Damage Per Second</h3>
                    <div class="grid-2">
                        <div class="stat-row highlight">
                            <span class="label">Total DPS</span>
                            <span class="val gold">{fmt(o.total_dps, 1)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label">Hit DPS</span>
                            <span class="val">{fmt(o.hit_dps, 1)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label">Average Hit</span>
                            <span class="val">{fmt(o.average_hit, 1)}</span>
                        </div>
                        {#if o.dot_dps > 0}
                            <div class="stat-row">
                                <span class="label">DoT DPS</span>
                                <span class="val">{fmt(o.dot_dps, 1)}</span>
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Per-element breakdown -->
                <div class="card">
                    <h3 class="card-title">Damage Breakdown</h3>
                    <div class="grid-2">
                        {#if o.phys_dps > 0}
                            <div class="stat-row">
                                <span class="label phys">Physical DPS</span>
                                <span class="val">{fmt(o.phys_dps, 1)}</span>
                            </div>
                        {/if}
                        {#if o.fire_dps > 0}
                            <div class="stat-row">
                                <span class="label fire">Fire DPS</span>
                                <span class="val">{fmt(o.fire_dps, 1)}</span>
                            </div>
                        {/if}
                        {#if o.cold_dps > 0}
                            <div class="stat-row">
                                <span class="label cold">Cold DPS</span>
                                <span class="val">{fmt(o.cold_dps, 1)}</span>
                            </div>
                        {/if}
                        {#if o.lightning_dps > 0}
                            <div class="stat-row">
                                <span class="label light">Lightning DPS</span>
                                <span class="val"
                                    >{fmt(o.lightning_dps, 1)}</span
                                >
                            </div>
                        {/if}
                        {#if o.chaos_dps > 0}
                            <div class="stat-row">
                                <span class="label chaos">Chaos DPS</span>
                                <span class="val">{fmt(o.chaos_dps, 1)}</span>
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Critical Strikes -->
                <div class="card">
                    <h3 class="card-title">Critical Strikes</h3>
                    <div class="grid-2">
                        <div class="stat-row">
                            <span class="label">Crit Chance</span>
                            <span class="val">{fmtPct(o.crit_chance)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label">Crit Multiplier</span>
                            <span class="val"
                                >{fmt(o.crit_multiplier * 100, 0)}%</span
                            >
                        </div>
                        <div class="stat-row">
                            <span class="label">Effective Crit</span>
                            <span class="val"
                                >{fmt(
                                    1 +
                                        (o.crit_chance / 100) *
                                            (o.crit_multiplier - 1),
                                    3,
                                )}×</span
                            >
                        </div>
                    </div>
                </div>

                <!-- Speed & Accuracy -->
                <div class="card">
                    <h3 class="card-title">Speed & Accuracy</h3>
                    <div class="grid-2">
                        {#if o.is_attack}
                            <div class="stat-row">
                                <span class="label">Attacks per Second</span>
                                <span class="val"
                                    >{o.attack_speed.toFixed(2)}</span
                                >
                            </div>
                            <div class="stat-row">
                                <span class="label">Hit Chance</span>
                                <span class="val">{fmtPct(o.hit_chance)}</span>
                            </div>
                        {:else}
                            <div class="stat-row">
                                <span class="label">Casts per Second</span>
                                <span class="val"
                                    >{o.cast_speed.toFixed(2)}</span
                                >
                            </div>
                            <div class="stat-row">
                                <span class="label">Hit Chance</span>
                                <span class="val">100% (spell)</span>
                            </div>
                        {/if}
                    </div>
                </div>

                <!-- Ailment DoT -->
                {#if o.bleed_dps > 0 || o.poison_dps > 0 || o.ignite_dps > 0}
                    <div class="card">
                        <h3 class="card-title">Ailments (per second)</h3>
                        <div class="grid-2">
                            {#if o.bleed_dps > 0}
                                <div class="stat-row">
                                    <span class="label phys">Bleed</span>
                                    <span class="val"
                                        >{fmt(o.bleed_dps, 1)}</span
                                    >
                                </div>
                            {/if}
                            {#if o.poison_dps > 0}
                                <div class="stat-row">
                                    <span class="label chaos">Poison</span>
                                    <span class="val"
                                        >{fmt(o.poison_dps, 1)}</span
                                    >
                                </div>
                            {/if}
                            {#if o.ignite_dps > 0}
                                <div class="stat-row">
                                    <span class="label fire">Ignite</span>
                                    <span class="val"
                                        >{fmt(o.ignite_dps, 1)}</span
                                    >
                                </div>
                            {/if}
                        </div>
                    </div>
                {/if}
            {/if}
        {:else if activeTab === "defence"}
            <div class="section-header">
                <h1 class="section-title">Defence</h1>
            </div>

            {#if d}
                <!-- Resources -->
                <div class="card">
                    <h3 class="card-title">Resources</h3>
                    <div class="grid-2">
                        <div class="stat-row">
                            <span class="label life">Life</span>
                            <span class="val life-val">{fmt(d.life)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label mana">Mana</span>
                            <span class="val mana-val">{fmt(d.mana)}</span>
                        </div>
                        {#if d.energy_shield > 0}
                            <div class="stat-row">
                                <span class="label es">Energy Shield</span>
                                <span class="val es-val"
                                    >{fmt(d.energy_shield)}</span
                                >
                            </div>
                        {/if}
                        {#if d.ward > 0}
                            <div class="stat-row">
                                <span class="label">Ward</span>
                                <span class="val">{fmt(d.ward)}</span>
                            </div>
                        {/if}
                        <div class="stat-row">
                            <span class="label">Unreserved Life</span>
                            <span class="val">{fmt(d.life_unreserved)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label">Unreserved Mana</span>
                            <span class="val">{fmt(d.mana_unreserved)}</span>
                        </div>
                    </div>
                </div>

                <!-- Regen / Recharge -->
                <div class="card">
                    <h3 class="card-title">Recovery (per second)</h3>
                    <div class="grid-2">
                        <div class="stat-row">
                            <span class="label life">Life Regen</span>
                            <span class="val">{fmt(d.life_regen, 1)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="label mana">Mana Regen</span>
                            <span class="val">{fmt(d.mana_regen, 1)}</span>
                        </div>
                        {#if d.es_regen > 0 || d.energy_shield > 0}
                            <div class="stat-row">
                                <span class="label es">ES Regen</span>
                                <span class="val">{fmt(d.es_regen, 1)}</span>
                            </div>
                            <div class="stat-row">
                                <span class="label es">ES Recharge</span>
                                <span class="val"
                                    >{fmt(d.es_recharge, 1)} (delay {d.es_recharge_delay.toFixed(
                                        2,
                                    )}s)</span
                                >
                            </div>
                        {/if}
                        <div class="stat-row">
                            <span class="label life">Max Life Leech/s</span>
                            <span class="val"
                                >{fmt(d.life_leech_rate_max, 1)}</span
                            >
                        </div>
                        <div class="stat-row">
                            <span class="label mana">Max Mana Leech/s</span>
                            <span class="val"
                                >{fmt(d.mana_leech_rate_max, 1)}</span
                            >
                        </div>
                    </div>
                </div>

                <!-- Mitigation -->
                <div class="card">
                    <h3 class="card-title">Mitigation</h3>
                    <div class="grid-2">
                        {#if d.armour > 0}
                            <div class="stat-row">
                                <span class="label">Armour</span>
                                <span class="val">{fmt(d.armour)}</span>
                            </div>
                        {/if}
                        {#if d.evasion > 0}
                            <div class="stat-row">
                                <span class="label">Evasion</span>
                                <span class="val">{fmt(d.evasion)}</span>
                            </div>
                        {/if}
                        {#if d.block_chance > 0}
                            <div class="stat-row">
                                <span class="label">Block Chance</span>
                                <span class="val">{fmtPct(d.block_chance)}</span
                                >
                            </div>
                        {/if}
                        {#if d.spell_block_chance > 0}
                            <div class="stat-row">
                                <span class="label">Spell Block</span>
                                <span class="val"
                                    >{fmtPct(d.spell_block_chance)}</span
                                >
                            </div>
                        {/if}
                        {#if d.spell_suppression > 0}
                            <div class="stat-row">
                                <span class="label">Spell Suppression</span>
                                <span class="val"
                                    >{fmtPct(d.spell_suppression)}</span
                                >
                            </div>
                        {/if}
                        {#if d.attack_dodge > 0}
                            <div class="stat-row">
                                <span class="label">Attack Dodge</span>
                                <span class="val">{fmtPct(d.attack_dodge)}</span
                                >
                            </div>
                        {/if}
                        {#if d.spell_dodge > 0}
                            <div class="stat-row">
                                <span class="label">Spell Dodge</span>
                                <span class="val">{fmtPct(d.spell_dodge)}</span>
                            </div>
                        {/if}
                        <div class="stat-row">
                            <span class="label">Movement Speed</span>
                            <span class="val"
                                >{fmtPct((d.movement_speed_mod - 1) * 100)} bonus</span
                            >
                        </div>
                    </div>
                </div>

                <!-- Resistances -->
                <div class="card">
                    <h3 class="card-title">Resistances</h3>
                    <table class="resist-table">
                        <thead>
                            <tr>
                                <th>Element</th>
                                <th>Effective</th>
                                <th>Cap</th>
                                <th>Overcap</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td class="res-name fire">Fire</td>
                                <td class="res-val {resistClass(d.fire_resist)}"
                                    >{d.fire_resist}%</td
                                >
                                <td class="res-dim">{d.fire_resist_cap}%</td>
                                <td class="res-dim"
                                    >{d.fire_resist_overcap > 0
                                        ? `+${d.fire_resist_overcap}%`
                                        : "—"}</td
                                >
                            </tr>
                            <tr>
                                <td class="res-name cold">Cold</td>
                                <td class="res-val {resistClass(d.cold_resist)}"
                                    >{d.cold_resist}%</td
                                >
                                <td class="res-dim">{d.cold_resist_cap}%</td>
                                <td class="res-dim"
                                    >{d.cold_resist_overcap > 0
                                        ? `+${d.cold_resist_overcap}%`
                                        : "—"}</td
                                >
                            </tr>
                            <tr>
                                <td class="res-name light">Lightning</td>
                                <td
                                    class="res-val {resistClass(
                                        d.lightning_resist,
                                    )}">{d.lightning_resist}%</td
                                >
                                <td class="res-dim"
                                    >{d.lightning_resist_cap}%</td
                                >
                                <td class="res-dim"
                                    >{d.lightning_resist_overcap > 0
                                        ? `+${d.lightning_resist_overcap}%`
                                        : "—"}</td
                                >
                            </tr>
                            <tr>
                                <td class="res-name chaos">Chaos</td>
                                <td
                                    class="res-val {resistClass(
                                        d.chaos_resist,
                                    )}">{d.chaos_resist}%</td
                                >
                                <td class="res-dim">{d.chaos_resist_cap}%</td>
                                <td class="res-dim"
                                    >{d.chaos_resist_overcap > 0
                                        ? `+${d.chaos_resist_overcap}%`
                                        : "—"}</td
                                >
                            </tr>
                        </tbody>
                    </table>
                </div>
            {/if}
        {/if}
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        overflow: hidden;
        background-color: #0a0a0a;
    }

    .calcs-page {
        display: flex;
        width: 100vw;
        height: 100vh;
        font-family: sans-serif;
        color: #dfcf99;
        background: #0a0a0a;
    }

    /* Left nav panel */
    .nav-panel {
        width: 180px;
        min-width: 180px;
        background: #111;
        border-right: 1px solid #2a2a2a;
        display: flex;
        flex-direction: column;
        padding: 16px;
        box-sizing: border-box;
        gap: 8px;
    }

    .btn-back {
        background: none;
        border: 1px solid #3a3730;
        border-radius: 6px;
        color: #e0d6c2;
        cursor: pointer;
        padding: 0.45em 0.8em;
        font-size: 0.85rem;
        text-align: left;
        width: 100%;
        transition:
            border-color 0.15s,
            background 0.15s;
    }
    .btn-back:hover {
        border-color: #c8a96e;
        background: #22201a;
    }

    .panel-title {
        font-size: 1em;
        color: #fff;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        margin: 8px 0 4px;
    }

    .tab-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .tab-btn {
        background: #1a1a1e;
        border: 1px solid #2a2a2a;
        border-radius: 5px;
        color: #aaa;
        cursor: pointer;
        padding: 0.5em 0.8em;
        font-size: 0.88rem;
        text-align: left;
        transition:
            border-color 0.15s,
            color 0.15s,
            background 0.15s;
        width: 100%;
    }
    .tab-btn:hover {
        border-color: #c8a96e;
        color: #dfcf99;
        background: #22201a;
    }
    .tab-btn.active {
        border-color: #c8a95e;
        background: #1d1a10;
        color: #c8a95e;
    }

    /* Main content */
    .content {
        flex: 1;
        overflow-y: auto;
        padding: 24px;
        box-sizing: border-box;
    }

    .section-header {
        margin-bottom: 20px;
    }

    .section-title {
        font-size: 1.5em;
        color: #fff;
        margin: 0 0 4px;
    }

    .hint {
        color: #666;
        font-size: 0.88em;
        margin: 0;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        gap: 8px;
        color: #555;
        font-size: 0.95em;
        padding-top: 40px;
    }

    /* Cards */
    .card {
        background: #111;
        border: 1px solid #2a2a2a;
        border-radius: 8px;
        padding: 16px;
        margin-bottom: 16px;
    }

    .card-title {
        font-size: 0.78em;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #666;
        margin: 0 0 12px;
        font-weight: 600;
    }

    .grid-2 {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 6px 16px;
    }

    .stat-row {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        padding: 3px 0;
        border-bottom: 1px solid #1a1a1a;
    }

    .stat-row.highlight {
        border-bottom: 1px solid #2a2a20;
    }

    .label {
        font-size: 0.83em;
        color: #999;
    }

    .val {
        font-size: 0.95em;
        font-weight: bold;
        color: #c8c0a0;
    }

    .gold {
        color: #c8a95e;
        font-size: 1.05em;
    }
    .life-val {
        color: #e05030;
    }
    .mana-val {
        color: #5b9bd5;
    }
    .es-val {
        color: #9ecfde;
    }

    /* Element colours for labels */
    .label.phys {
        color: #c0b888;
    }
    .label.fire {
        color: #e05030;
    }
    .label.cold {
        color: #5bb8d4;
    }
    .label.light {
        color: #d4b83c;
    }
    .label.chaos {
        color: #d44be0;
    }
    .label.life {
        color: #e05030;
    }
    .label.mana {
        color: #5b9bd5;
    }
    .label.es {
        color: #9ecfde;
    }

    /* Resistance table */
    .resist-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.88em;
    }

    .resist-table th {
        text-align: left;
        color: #555;
        font-weight: 600;
        font-size: 0.78em;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        padding: 4px 8px;
        border-bottom: 1px solid #2a2a2a;
    }

    .resist-table td {
        padding: 6px 8px;
        border-bottom: 1px solid #1a1a1a;
    }

    .res-name {
        font-weight: 600;
    }
    .res-name.fire {
        color: #e05030;
    }
    .res-name.cold {
        color: #5bb8d4;
    }
    .res-name.light {
        color: #d4b83c;
    }
    .res-name.chaos {
        color: #d44be0;
    }

    .res-val {
        font-weight: bold;
    }
    .res-val.neg {
        color: #e05050;
    }
    .res-val.ok {
        color: #d4c080;
    }
    .res-val.cap {
        color: #5cb85c;
    }
    .res-dim {
        color: #666;
    }
</style>
