<script lang="ts">
    import { onMount } from "svelte";
    import { commands } from "../bindings";
    import type {
        GemSummary,
        SkillGroup,
        GemInstance,
        SupportCompatEntry,
    } from "../bindings";

    let allGems = $state<GemSummary[]>([]);
    let skillGroups = $state<SkillGroup[]>([]);
    let newGroupLabel = $state("");
    let gemSearches = $state<Record<number, string>>({});
    let selectedGemKey = $state<string | null>(null);
    let loading = $state(true);

    /** The currently selected gem instance (derived from key) */
    let selectedGem = $derived.by(() => {
        if (!selectedGemKey) return null;
        const [gid, gi] = selectedGemKey.split("-");
        const group = skillGroups.find((g) => g.id === Number(gid));
        if (!group) return null;
        const gem = group.gems[Number(gi)];
        return gem ?? null;
    });

    /** Color of the selected gem */
    let selectedGemColor = $derived(
        selectedGem ? gemColor(selectedGem.gem_id) : "white",
    );

    onMount(async () => {
        const [gemsResult, groupsResult] = await Promise.all([
            commands.getGemList(),
            commands.getSkillGroups(),
        ]);
        if (gemsResult.status === "ok") allGems = gemsResult.data;
        if (groupsResult.status === "ok") skillGroups = groupsResult.data;
        loading = false;
    });

    function filteredGems(search: string): GemSummary[] {
        if (!search) return allGems;
        const lower = search.toLowerCase();
        return allGems.filter((g) => g.name.toLowerCase().includes(lower));
    }

    async function createGroup() {
        const label = newGroupLabel.trim() || "Skill Group";
        const result = await commands.createSkillGroup(label);
        if (result.status === "ok") {
            skillGroups = [...skillGroups, result.data];
            newGroupLabel = "";
        }
    }

    async function deleteGroup(id: number) {
        const result = await commands.deleteSkillGroup(id);
        if (result.status === "ok") {
            skillGroups = skillGroups.filter((g) => g.id !== id);
        }
    }

    async function addGem(groupId: number, gemId: string) {
        const result = await commands.addGemToGroup(groupId, gemId);
        if (result.status === "ok") {
            skillGroups = skillGroups.map((g) =>
                g.id === groupId ? result.data : g,
            );
            gemSearches[groupId] = "";
        }
    }

    async function removeGem(groupId: number, gemIndex: number) {
        const result = await commands.removeGemFromGroup(groupId, gemIndex);
        if (result.status === "ok") {
            skillGroups = skillGroups.map((g) =>
                g.id === groupId ? result.data : g,
            );
            // Clear selection if we removed the selected gem
            if (selectedGemKey === `${groupId}-${gemIndex}`) {
                selectedGemKey = null;
            }
        }
    }

    async function updateGemLevelQuality(
        groupId: number,
        gemIndex: number,
        level: number,
        quality: number,
    ) {
        const result = await commands.updateGemLevelQuality(
            groupId,
            gemIndex,
            level,
            quality,
        );
        if (result.status === "ok") {
            skillGroups = skillGroups.map((g) =>
                g.id === groupId ? result.data : g,
            );
        }
    }

    function gemColor(gemId: string): string {
        return allGems.find((g) => g.id === gemId)?.color ?? "white";
    }

    function gemDescription(gemId: string): string | undefined {
        return allGems.find((g) => g.id === gemId)?.description ?? undefined;
    }

    function gemTagString(gemId: string): string | undefined {
        return allGems.find((g) => g.id === gemId)?.tag_string ?? undefined;
    }

    function selectGem(groupId: number, idx: number) {
        const key = `${groupId}-${idx}`;
        selectedGemKey = selectedGemKey === key ? null : key;
    }

    /** Check if a support is incompatible with any active in the group */
    function getIncompatibleActives(
        group: SkillGroup,
        supportGemId: string,
    ): string[] {
        return group.compatibility
            .filter((c) => c.support_gem_id === supportGemId && !c.compatible)
            .map((c) => {
                const gem = group.gems.find(
                    (g) => g.gem_id === c.active_gem_id,
                );
                return gem?.name ?? c.active_gem_id;
            });
    }

    /** Format a stat key for display */
    function formatStatName(key: string): string {
        return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
    }

    /** Get notable properties to display */
    function getGemProperties(
        gem: GemInstance,
    ): { label: string; value: string }[] {
        const props: { label: string; value: string }[] = [];
        if (gem.mana_cost != null)
            props.push({ label: "Mana Cost", value: gem.mana_cost.toString() });
        if (gem.crit_chance != null)
            props.push({ label: "Crit Chance", value: `${gem.crit_chance}%` });
        if (gem.damage_effectiveness != null)
            props.push({
                label: "Damage Effectiveness",
                value: `${Math.round(gem.damage_effectiveness * 100)}%`,
            });
        if (gem.mana_multiplier != null)
            props.push({
                label: "Mana Multiplier",
                value: `${Math.round(gem.mana_multiplier)}%`,
            });
        if (gem.cooldown != null)
            props.push({ label: "Cooldown", value: `${gem.cooldown}s` });
        if (gem.attack_speed_multiplier != null)
            props.push({
                label: "Attack Speed Mult",
                value: `${gem.attack_speed_multiplier}%`,
            });
        return props;
    }
</script>

{#if loading}
    <p class="loading">Loading gems...</p>
{:else}
    <div class="skills-layout">
        <!-- Left: Skill Groups -->
        <div class="groups-column">
            <div class="create-group">
                <input
                    type="text"
                    bind:value={newGroupLabel}
                    placeholder="New group name..."
                    onkeydown={(e) => e.key === "Enter" && createGroup()}
                />
                <button class="btn-create" onclick={createGroup}
                    >+ Add Group</button
                >
            </div>

            {#if skillGroups.length === 0}
                <div class="empty-state">
                    <p>No skill groups yet.</p>
                    <p class="hint">
                        Create a group and add gems to build your skill setup.
                    </p>
                </div>
            {:else}
                {#each skillGroups as group (group.id)}
                    <div class="group-card">
                        <div class="group-header">
                            <h3>{group.label}</h3>
                            <button
                                class="btn-delete"
                                onclick={() => deleteGroup(group.id)}
                                title="Delete group">✕</button
                            >
                        </div>

                        <ul class="gem-list">
                            {#each group.gems as gem, idx (idx)}
                                {@const incompatible = gem.is_support
                                    ? getIncompatibleActives(group, gem.gem_id)
                                    : []}
                                {@const hasWarning = incompatible.length > 0}
                                {@const isSelected =
                                    selectedGemKey === `${group.id}-${idx}`}
                                <li class="gem-entry">
                                    <button
                                        class="gem-item gem-color-{gemColor(
                                            gem.gem_id,
                                        )}"
                                        class:gem-incompatible={hasWarning}
                                        class:gem-selected={isSelected}
                                        onclick={() => selectGem(group.id, idx)}
                                    >
                                        <span class="gem-name">{gem.name}</span>
                                        <label class="gem-lq" title="Level">
                                            Lv
                                            <input
                                                type="number"
                                                class="input-lq"
                                                min="1"
                                                max="40"
                                                value={gem.level}
                                                onclick={(e) =>
                                                    e.stopPropagation()}
                                                onchange={(e) =>
                                                    updateGemLevelQuality(
                                                        group.id,
                                                        idx,
                                                        parseInt(
                                                            (
                                                                e.target as HTMLInputElement
                                                            ).value,
                                                        ) || 1,
                                                        gem.quality,
                                                    )}
                                            />
                                        </label>
                                        <label class="gem-lq" title="Quality">
                                            Q
                                            <input
                                                type="number"
                                                class="input-lq"
                                                min="0"
                                                max="100"
                                                value={gem.quality}
                                                onclick={(e) =>
                                                    e.stopPropagation()}
                                                onchange={(e) =>
                                                    updateGemLevelQuality(
                                                        group.id,
                                                        idx,
                                                        gem.level,
                                                        parseInt(
                                                            (
                                                                e.target as HTMLInputElement
                                                            ).value,
                                                        ) || 0,
                                                    )}
                                            />
                                        </label>
                                        <span class="gem-type"
                                            >{gem.is_support
                                                ? "Support"
                                                : "Active"}</span
                                        >
                                        <span
                                            class="btn-remove-gem"
                                            role="button"
                                            tabindex="0"
                                            title="Remove gem"
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                removeGem(group.id, idx);
                                            }}
                                            onkeydown={(e) => {
                                                if (e.key === "Enter") {
                                                    e.stopPropagation();
                                                    removeGem(group.id, idx);
                                                }
                                            }}>✕</span
                                        >
                                    </button>
                                    {#if hasWarning}
                                        <div class="compat-warning">
                                            ⚠ Cannot support: {incompatible.join(
                                                ", ",
                                            )}
                                        </div>
                                    {/if}
                                </li>
                            {/each}
                        </ul>

                        <div class="gem-selector">
                            <input
                                type="text"
                                bind:value={gemSearches[group.id]}
                                placeholder="Search gems..."
                            />
                            {#if gemSearches[group.id]}
                                <ul class="gem-dropdown">
                                    {#each filteredGems(gemSearches[group.id] ?? "").slice(0, 20) as gem (gem.id)}
                                        <li>
                                            <button
                                                class="gem-option gem-text-{gem.color}"
                                                title={gem.description ??
                                                    undefined}
                                                onclick={() =>
                                                    addGem(group.id, gem.id)}
                                            >
                                                <span>{gem.name}</span>
                                                <span class="gem-tag"
                                                    >{gem.is_support
                                                        ? "Support"
                                                        : "Active"}</span
                                                >
                                            </button>
                                        </li>
                                    {/each}
                                </ul>
                            {/if}
                        </div>
                    </div>
                {/each}
            {/if}
        </div>

        <!-- Right: Gem Info Panel (PoE-style box) -->
        <div class="info-column">
            {#if selectedGem}
                <div class="gem-box gem-box-{selectedGemColor}">
                    <!-- Header -->
                    <div class="gem-box-header gem-header-{selectedGemColor}">
                        <span class="gem-box-name">{selectedGem.name}</span>
                    </div>

                    <!-- Tags -->
                    {#if gemTagString(selectedGem.gem_id)}
                        <div class="gem-box-tags">
                            {gemTagString(selectedGem.gem_id)}
                        </div>
                    {/if}

                    <!-- Level & Quality -->
                    <div class="gem-box-section">
                        <div class="gem-box-row">
                            <span>Level</span>
                            <span class="gem-box-val">{selectedGem.level}</span>
                        </div>
                        {#if selectedGem.quality > 0}
                            <div class="gem-box-row quality-row">
                                <span>Quality</span>
                                <span class="gem-box-val quality-val"
                                    >+{selectedGem.quality}%</span
                                >
                            </div>
                        {/if}
                    </div>

                    <div class="gem-box-separator"></div>

                    <!-- Properties -->
                    {#if getGemProperties(selectedGem).length > 0}
                        <div class="gem-box-section">
                            {#each getGemProperties(selectedGem) as prop}
                                <div class="gem-box-row">
                                    <span>{prop.label}</span>
                                    <span class="gem-box-val">{prop.value}</span
                                    >
                                </div>
                            {/each}
                        </div>
                        <div class="gem-box-separator"></div>
                    {/if}

                    <!-- Description -->
                    {#if gemDescription(selectedGem.gem_id)}
                        <div class="gem-box-desc">
                            {gemDescription(selectedGem.gem_id)}
                        </div>
                        <div class="gem-box-separator"></div>
                    {/if}

                    <!-- Stats -->
                    {#if Object.keys(selectedGem.stats).length > 0}
                        <div class="gem-box-stats">
                            {#each Object.entries(selectedGem.stats) as [key, value]}
                                <div class="gem-box-stat">
                                    <span>{formatStatName(key)}: </span>
                                    <span class="gem-box-stat-val">
                                        {typeof value === "number" &&
                                        !Number.isInteger(value)
                                            ? value.toFixed(1)
                                            : value}
                                    </span>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {:else}
                <div class="info-placeholder">
                    <p>Click a gem to view its details</p>
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    /* ── Layout ── */
    .skills-layout {
        display: flex;
        gap: 1.5rem;
        width: 100%;
        align-items: flex-start;
    }

    .groups-column {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .info-column {
        width: 320px;
        min-width: 320px;
        position: sticky;
        top: 1.5rem;
    }

    .loading {
        color: #8a8578;
        text-align: center;
        padding: 2rem;
    }

    /* ── Create Group ── */
    .create-group {
        display: flex;
        gap: 0.5rem;
    }

    .create-group input {
        flex: 1;
        padding: 0.5em 0.75em;
        background: #1a1a1e;
        border: 1px solid #3a3730;
        border-radius: 4px;
        color: #e0d6c2;
        font-size: 0.9rem;
    }

    .create-group input::placeholder {
        color: #5a5448;
    }

    .btn-create {
        padding: 0.5em 1em;
        background: linear-gradient(135deg, #c8a95e, #a38638);
        color: #0e0e10;
        border: none;
        border-radius: 4px;
        font-weight: 600;
        cursor: pointer;
        white-space: nowrap;
    }

    .btn-create:hover {
        filter: brightness(1.1);
    }

    .empty-state {
        text-align: center;
        padding: 2rem 0;
        color: #5a5448;
    }

    .hint {
        font-size: 0.85rem;
    }

    /* ── Group Card ── */
    .group-card {
        background: #16161a;
        border: 1px solid #2a2723;
        border-radius: 6px;
        padding: 1rem;
    }

    .group-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
    }

    .group-header h3 {
        margin: 0;
        font-size: 1rem;
        color: #c8a95e;
    }

    .btn-delete {
        background: none;
        border: 1px solid #3a3730;
        color: #8a8578;
        cursor: pointer;
        border-radius: 4px;
        padding: 0.2em 0.5em;
        font-size: 0.85rem;
        line-height: 1;
    }

    .btn-delete:hover {
        color: #e05555;
        border-color: #e05555;
    }

    /* ── Gem List ── */
    .gem-list {
        list-style: none;
        margin: 0 0 0.75rem;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .gem-entry {
        display: flex;
        flex-direction: column;
    }

    .gem-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.35em 0.5em;
        background: #1e1e22;
        border-radius: 4px;
        border: 1px solid transparent;
        border-left: 3px solid #aaa;
        cursor: pointer;
        transition:
            background 0.1s,
            border-color 0.1s;
        text-align: left;
        font: inherit;
        color: inherit;
        width: 100%;
        box-sizing: border-box;
    }

    .gem-item:hover {
        background: #24242a;
    }

    .gem-selected {
        background: #1a1e2a !important;
        border-color: #4a6090;
    }

    .gem-incompatible {
        border-left-color: #e05555 !important;
        background: #221a1a;
    }

    .gem-color-red {
        border-left-color: #c85e5e;
    }
    .gem-color-green {
        border-left-color: #5ec85e;
    }
    .gem-color-blue {
        border-left-color: #5e8ec8;
    }
    .gem-color-white {
        border-left-color: #c0c0c0;
    }

    .gem-name {
        flex: 1;
        color: #e0d6c2;
        font-size: 0.9rem;
    }

    .gem-lq {
        display: flex;
        align-items: center;
        gap: 0.2em;
        font-size: 0.72rem;
        color: #8a8578;
        white-space: nowrap;
    }

    .input-lq {
        width: 3em;
        padding: 0.1em 0.25em;
        background: #1a1a1e;
        border: 1px solid #3a3730;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.78rem;
        text-align: center;
        -moz-appearance: textfield;
        appearance: textfield;
    }

    .input-lq::-webkit-inner-spin-button,
    .input-lq::-webkit-outer-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }

    .input-lq:focus {
        outline: none;
        border-color: #c8a95e;
    }

    .gem-type {
        font-size: 0.75rem;
        color: #6a6458;
    }

    .btn-remove-gem {
        color: #5a5448;
        cursor: pointer;
        font-size: 0.8rem;
        padding: 0.1em 0.3em;
        line-height: 1;
    }

    .btn-remove-gem:hover {
        color: #e05555;
    }

    .compat-warning {
        background: #2a1a1a;
        color: #e08080;
        font-size: 0.78rem;
        padding: 0.25em 0.75em;
        border-radius: 0 0 4px 4px;
        border-left: 3px solid #e05555;
    }

    /* ── Gem Selector (search dropdown) ── */
    .gem-selector {
        position: relative;
    }

    .gem-selector input {
        width: 100%;
        padding: 0.4em 0.6em;
        background: #1a1a1e;
        border: 1px solid #3a3730;
        border-radius: 4px;
        color: #e0d6c2;
        font-size: 0.85rem;
        box-sizing: border-box;
    }

    .gem-selector input::placeholder {
        color: #5a5448;
    }

    .gem-dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        max-height: 240px;
        overflow-y: auto;
        background: #1a1a1e;
        border: 1px solid #3a3730;
        border-top: none;
        border-radius: 0 0 4px 4px;
        list-style: none;
        margin: 0;
        padding: 0;
        z-index: 20;
    }

    .gem-option {
        display: flex;
        justify-content: space-between;
        width: 100%;
        padding: 0.4em 0.6em;
        background: none;
        border: none;
        color: #e0d6c2;
        cursor: pointer;
        font-size: 0.85rem;
        text-align: left;
    }

    .gem-option:hover {
        background: #2a2a2e;
    }

    .gem-text-red {
        color: #e08080;
    }
    .gem-text-green {
        color: #80e080;
    }
    .gem-text-blue {
        color: #8ab4e0;
    }
    .gem-text-white {
        color: #d0d0d0;
    }

    .gem-tag {
        font-size: 0.72rem;
        color: #5a5448;
    }

    /* ── PoE-style Gem Info Box ── */
    .info-placeholder {
        text-align: center;
        color: #5a5448;
        padding: 3rem 1rem;
        border: 1px dashed #2a2723;
        border-radius: 6px;
    }

    .gem-box {
        background: #0c0b10;
        border: 1px solid #3a3632;
        border-radius: 2px;
        overflow: hidden;
        font-size: 0.85rem;
        line-height: 1.5;
    }

    .gem-box-header {
        padding: 0.6em 0.8em;
        text-align: center;
        border-bottom: 1px solid #3a3632;
    }

    .gem-header-red {
        background: linear-gradient(180deg, #3a1818 0%, #1a0c0c 100%);
    }
    .gem-header-green {
        background: linear-gradient(180deg, #183a18 0%, #0c1a0c 100%);
    }
    .gem-header-blue {
        background: linear-gradient(180deg, #18183a 0%, #0c0c1a 100%);
    }
    .gem-header-white {
        background: linear-gradient(180deg, #2a2a2e 0%, #16161a 100%);
    }

    .gem-box-name {
        font-size: 1.05rem;
        font-weight: 600;
        letter-spacing: 0.02em;
    }

    .gem-box-red .gem-box-name {
        color: #e08080;
    }
    .gem-box-green .gem-box-name {
        color: #80e080;
    }
    .gem-box-blue .gem-box-name {
        color: #8ab4e0;
    }
    .gem-box-white .gem-box-name {
        color: #d0d0d0;
    }

    .gem-box-tags {
        text-align: center;
        color: #8a8578;
        font-size: 0.78rem;
        padding: 0.3em 0.8em;
        border-bottom: 1px solid #2a2520;
    }

    .gem-box-section {
        padding: 0.4em 0.8em;
    }

    .gem-box-row {
        display: flex;
        justify-content: space-between;
        padding: 0.15em 0;
        color: #8a8578;
    }

    .gem-box-val {
        color: #e0d6c2;
    }

    .quality-row {
        color: #5e8ec8;
    }

    .quality-val {
        color: #8ab4e0;
    }

    .gem-box-separator {
        height: 1px;
        background: linear-gradient(
            90deg,
            transparent 0%,
            #3a3632 30%,
            #3a3632 70%,
            transparent 100%
        );
        margin: 0 0.5em;
    }

    .gem-box-desc {
        padding: 0.5em 0.8em;
        color: #af6025;
        font-style: italic;
        font-size: 0.82rem;
        line-height: 1.45;
    }

    .gem-box-stats {
        padding: 0.5em 0.8em;
    }

    .gem-box-stat {
        color: #7f7f7f;
        font-size: 0.82rem;
        padding: 0.1em 0;
    }

    .gem-box-stat-val {
        color: #e0d6c2;
    }
</style>
