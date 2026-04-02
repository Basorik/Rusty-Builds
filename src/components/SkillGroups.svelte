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
    let expandedGem = $state<string | null>(null);
    let loading = $state(true);

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
        }
    }

    function gemColor(gemId: string): string {
        return allGems.find((g) => g.id === gemId)?.color ?? "white";
    }

    function gemDescription(gemId: string): string | undefined {
        return allGems.find((g) => g.id === gemId)?.description ?? undefined;
    }

    function toggleExpanded(gemId: string) {
        expandedGem = expandedGem === gemId ? null : gemId;
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

    /** Format a stat key for display — replace underscores, title-case, trim internal IDs */
    function formatStatName(key: string): string {
        return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
    }

    /** Get notable stats to display for a gem (mana cost, crit, damage eff, etc.) */
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
    <div class="skill-groups">
        <div class="create-group">
            <input
                type="text"
                bind:value={newGroupLabel}
                placeholder="New group name..."
                onkeydown={(e) => e.key === "Enter" && createGroup()}
            />
            <button class="btn-create" onclick={createGroup}>+ Add Group</button
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
                            <li class="gem-entry">
                                <div
                                    class="gem-item gem-color-{gemColor(
                                        gem.gem_id,
                                    )}"
                                    class:gem-incompatible={hasWarning}
                                >
                                    <button
                                        class="gem-expand"
                                        onclick={() =>
                                            toggleExpanded(
                                                `${group.id}-${idx}`,
                                            )}
                                    >
                                        {expandedGem === `${group.id}-${idx}`
                                            ? "▾"
                                            : "▸"}
                                    </button>
                                    <span class="gem-name">{gem.name}</span>
                                    <span class="gem-level">Lv {gem.level}</span
                                    >
                                    <span class="gem-type"
                                        >{gem.is_support
                                            ? "Support"
                                            : "Active"}</span
                                    >
                                    <button
                                        class="btn-remove-gem"
                                        onclick={() => removeGem(group.id, idx)}
                                        title="Remove gem">✕</button
                                    >
                                </div>

                                {#if hasWarning}
                                    <div class="compat-warning">
                                        ⚠ Cannot support: {incompatible.join(
                                            ", ",
                                        )}
                                    </div>
                                {/if}

                                {#if expandedGem === `${group.id}-${idx}`}
                                    <div class="gem-details">
                                        {#if gemDescription(gem.gem_id)}
                                            <p class="gem-desc">
                                                {gemDescription(gem.gem_id)}
                                            </p>
                                        {/if}

                                        {#if getGemProperties(gem).length > 0}
                                            <div class="gem-props">
                                                {#each getGemProperties(gem) as prop}
                                                    <div class="prop-row">
                                                        <span class="prop-label"
                                                            >{prop.label}</span
                                                        >
                                                        <span class="prop-value"
                                                            >{prop.value}</span
                                                        >
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}

                                        {#if Object.keys(gem.stats).length > 0}
                                            <div class="gem-stats">
                                                <h4>
                                                    Stats at Level {gem.level}
                                                </h4>
                                                {#each Object.entries(gem.stats) as [key, value]}
                                                    <div class="stat-row">
                                                        <span class="stat-name"
                                                            >{formatStatName(
                                                                key,
                                                            )}</span
                                                        >
                                                        <span class="stat-value"
                                                            >{typeof value ===
                                                                "number" &&
                                                            !Number.isInteger(
                                                                value,
                                                            )
                                                                ? value.toFixed(
                                                                      1,
                                                                  )
                                                                : value}</span
                                                        >
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}
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
                                            title={gem.description ?? undefined}
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
{/if}

<style>
    .skill-groups {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
    }

    .loading {
        color: #8a8578;
        text-align: center;
        padding: 2rem;
    }

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
        border-left: 3px solid #aaa;
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

    .gem-expand {
        background: none;
        border: none;
        color: #6a6458;
        cursor: pointer;
        font-size: 0.8rem;
        padding: 0;
        width: 1em;
        text-align: center;
    }

    .gem-name {
        flex: 1;
        color: #e0d6c2;
        font-size: 0.9rem;
    }

    .gem-level {
        font-size: 0.72rem;
        color: #8a8578;
    }

    .gem-type {
        font-size: 0.75rem;
        color: #6a6458;
    }

    .btn-remove-gem {
        background: none;
        border: none;
        color: #5a5448;
        cursor: pointer;
        font-size: 0.8rem;
        padding: 0.1em 0.3em;
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

    .gem-details {
        background: #12121a;
        border: 1px solid #2a2723;
        border-top: none;
        border-radius: 0 0 4px 4px;
        padding: 0.6em 0.75em;
        margin-top: -1px;
    }

    .gem-desc {
        color: #8a8578;
        font-size: 0.8rem;
        font-style: italic;
        margin: 0 0 0.5em;
    }

    .gem-props {
        display: flex;
        flex-wrap: wrap;
        gap: 0.4em 1.2em;
        margin-bottom: 0.5em;
    }

    .prop-row {
        display: flex;
        gap: 0.3em;
        font-size: 0.8rem;
    }

    .prop-label {
        color: #8a8578;
    }

    .prop-value {
        color: #c8a95e;
    }

    .gem-stats h4 {
        color: #8a8578;
        font-size: 0.78rem;
        margin: 0.3em 0 0.2em;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .stat-row {
        display: flex;
        justify-content: space-between;
        font-size: 0.78rem;
        padding: 0.1em 0;
    }

    .stat-name {
        color: #a09888;
    }

    .stat-value {
        color: #e0d6c2;
        font-variant-numeric: tabular-nums;
    }

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
</style>
