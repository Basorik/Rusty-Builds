<script lang="ts">
    import Header from "../../components/Header.svelte";
    import Sidebar from "../../components/Sidebar.svelte";
    import { getBuildState } from "$lib/buildState.svelte";
    import { commands } from "../../bindings";
    import type {
        BaseCategory,
        BaseItemProps,
        BaseModGroups,
        CraftedItemSpec,
        CraftedModValue,
        ModTierInfo,
        StatModGroup,
    } from "../../bindings";

    const build = getBuildState();

    // ── Navigation ────────────────────────────────────────────────────────────
    let categories = $state<BaseCategory[]>([]);
    let selCatName = $state("");
    let selSubLabel = $state("");
    let selBase = $state("");

    let selCategory = $derived(categories.find(c => c.name === selCatName) ?? null);
    let selSubcategory = $derived(
        selCategory?.subcategories.find(s => s.label === selSubLabel) ?? null
    );

    // ── Base data ─────────────────────────────────────────────────────────────
    let baseProps = $state<BaseItemProps | null>(null);
    let baseModGroups = $state<BaseModGroups | null>(null);
    let loadingBase = $state(false);

    // ── Config ────────────────────────────────────────────────────────────────
    let itemName = $state("");
    let itemLevel = $state(100);
    let quality = $state(20);
    let influence = $state("");

    let overridePhysMin = $state<number | null>(null);
    let overridePhysMax = $state<number | null>(null);
    let overrideArmour = $state<number | null>(null);
    let overrideEvasion = $state<number | null>(null);
    let overrideES = $state<number | null>(null);

    // ── Mod selection ─────────────────────────────────────────────────────────
    // Record<groupIndex, { tierIdx, values[] }>
    type SelMap = Record<number, { tierIdx: number; values: number[] }>;
    let implicitSel = $state<SelMap>({});
    let prefixSel   = $state<SelMap>({});
    let suffixSel   = $state<SelMap>({});
    let craftedSel  = $state<SelMap>({});

    let filterPrefix  = $state("");
    let filterSuffix  = $state("");
    let filterCrafted = $state("");

    // Status
    let addStatus = $state("");
    let adding = $state(false);

    // ── Load category tree on mount ───────────────────────────────────────────
    $effect(() => {
        commands.getBaseCategories().then(res => {
            if (res.status === "ok") categories = res.data;
        });
    });

    // ── Navigation handlers ───────────────────────────────────────────────────
    function selectCategory(name: string) {
        selCatName = name;
        selSubLabel = "";
        selBase = "";
        baseProps = null;
        baseModGroups = null;
        clearSelections();
    }

    function selectSubcategory(label: string) {
        selSubLabel = label;
        selBase = "";
        baseProps = null;
        baseModGroups = null;
        clearSelections();
    }

    async function selectBase(name: string) {
        selBase = name;
        clearSelections();
        await loadBase(name, influence);
    }

    async function onInfluenceChange() {
        if (selBase) await loadBase(selBase, influence);
    }

    async function loadBase(name: string, inf: string) {
        loadingBase = true;
        baseProps = null;
        baseModGroups = null;
        const [propsRes, modsRes] = await Promise.all([
            commands.getBaseItemProps(name),
            commands.getModsForBaseGrouped(name, inf || null),
        ]);
        loadingBase = false;
        if (propsRes.status === "ok") {
            const p = propsRes.data;
            baseProps = p;
            overridePhysMin = p.phys_damage_min;
            overridePhysMax = p.phys_damage_max;
            overrideArmour  = p.armour_max;
            overrideEvasion = p.evasion_max;
            overrideES      = p.energy_shield_max;
        }
        if (modsRes.status === "ok") {
            baseModGroups = modsRes.data;
            // Auto-populate all implicits at tier 0 (best/only tier).
            const newSel: SelMap = {};
            modsRes.data.implicits.forEach((group, i) => {
                if (group.tiers.length > 0) {
                    const t = group.tiers[0];
                    newSel[i] = { tierIdx: 0, values: t.stats.map(s => midpoint(s.min, s.max)) };
                }
            });
            implicitSel = newSel;
        }
    }

    function clearSelections() {
        implicitSel = {};
        prefixSel   = {};
        suffixSel   = {};
        craftedSel  = {};
        overridePhysMin = null;
        overridePhysMax = null;
        overrideArmour  = null;
        overrideEvasion = null;
        overrideES      = null;
    }

    // ── Mod selection helpers ─────────────────────────────────────────────────
    function midpoint(min: number, max: number): number {
        return min === max ? min : Math.round((min + max) / 2);
    }

    function setTierFor(
        sel: SelMap,
        groups: StatModGroup[],
        gi: number,
        tierIdx: number
    ): SelMap {
        if (tierIdx < 0) {
            const next = { ...sel };
            delete next[gi];
            return next;
        }
        const tier = groups[gi].tiers[tierIdx];
        return { ...sel, [gi]: { tierIdx, values: tier.stats.map(s => midpoint(s.min, s.max)) } };
    }

    function setValueFor(sel: SelMap, gi: number, si: number, value: number): SelMap {
        const existing = sel[gi];
        if (!existing) return sel;
        const values = [...existing.values];
        values[si] = value;
        return { ...sel, [gi]: { ...existing, values } };
    }

    function countSel(sel: SelMap): number {
        return Object.keys(sel).length;
    }

    function tierLabel(tier: ModTierInfo): string {
        if (tier.stats.length === 0) return `T${tier.tier} (lv${tier.required_level})`;
        const s = tier.stats[0];
        const range = s.min === s.max
            ? String(Math.round(s.min))
            : `${Math.round(s.min)}–${Math.round(s.max)}`;
        return `T${tier.tier} (lv${tier.required_level}) ${range}`;
    }

    function filteredIndices(groups: StatModGroup[], filter: string): number[] {
        const q = filter.toLowerCase().trim();
        if (!q) return groups.map((_, i) => i);
        return groups.reduce<number[]>((acc, g, i) => {
            if (
                g.display_name.toLowerCase().includes(q) ||
                g.tiers.some(t => t.stats.some(s => s.stat_id.toLowerCase().includes(q)))
            ) {
                acc.push(i);
            }
            return acc;
        }, []);
    }

    function buildModValues(sel: SelMap, groups: StatModGroup[]): CraftedModValue[] {
        return Object.entries(sel).map(([idxStr, s]) => ({
            mod_id: groups[parseInt(idxStr)].tiers[s.tierIdx].mod_id,
            values: s.values,
        }));
    }

    // Fill '#' placeholders in a display template with actual values.
    function fillTemplate(name: string, values: number[]): string {
        let i = 0;
        return name.replace(/#/g, () => String(Math.round(values[i++] ?? 0)));
    }

    // ── Craft action ──────────────────────────────────────────────────────────
    async function addToBuild() {
        if (!selBase || !baseModGroups) return;
        adding = true;
        addStatus = "";
        const spec: CraftedItemSpec = {
            base_name: selBase,
            item_name: itemName.trim(),
            quality,
            item_level: itemLevel,
            base_phys_min: overridePhysMin,
            base_phys_max: overridePhysMax,
            base_armour:   overrideArmour,
            base_evasion:  overrideEvasion,
            base_energy_shield: overrideES,
            implicits: buildModValues(implicitSel, baseModGroups.implicits),
            prefixes:  buildModValues(prefixSel,   baseModGroups.prefixes),
            suffixes:  buildModValues(suffixSel,   baseModGroups.suffixes),
            crafted:   buildModValues(craftedSel,  baseModGroups.crafted),
            influence: influence || null,
        };
        const res = await commands.addCraftedItem(spec);
        if (res.status === "ok") {
            build.buildStats = res.data;
            const inv = await commands.getInventoryItems();
            if (inv.status === "ok") build.inventoryItems = inv.data;
            addStatus = `"${itemName.trim() || selBase}" added to inventory.`;
        } else {
            addStatus = `Error: ${res.error}`;
        }
        adding = false;
    }
</script>

<main class="craft-page">
    <Sidebar
        selectedCount={build.selectedCount}
        ascSelectedCount={build.ascSelectedCount}
        buildStats={build.buildStats}
    />
    <div class="page-body">
        <Header
            bind:characterClass={build.characterClass}
            bind:ascendancy={build.ascendancy}
            bind:bloodline={build.bloodline}
            bind:level={build.level}
        />
        <section class="content">

            <!-- ── Left: hierarchical base browser ─────────────────────── -->
            <div class="panel browser-panel">
                <!-- Category tabs -->
                <div class="cat-tabs">
                    {#each categories as cat}
                        <button
                            class="cat-tab"
                            class:active={selCatName === cat.name}
                            onclick={() => selectCategory(cat.name)}
                        >{cat.name}</button>
                    {/each}
                </div>

                {#if selCategory}
                    <!-- Subcategory list -->
                    <div class="subcat-list">
                        {#each selCategory.subcategories as sub}
                            <button
                                class="subcat-btn"
                                class:active={selSubLabel === sub.label}
                                onclick={() => selectSubcategory(sub.label)}
                            >{sub.label}</button>
                        {/each}
                    </div>
                {/if}

                {#if selSubcategory}
                    <!-- Base list -->
                    <div class="base-sep"></div>
                    <ul class="base-list">
                        {#each selSubcategory.bases as base}
                            <li class="base-item" class:active={selBase === base.name}>
                                <button class="base-btn" onclick={() => selectBase(base.name)}>
                                    <span class="base-name">{base.name}</span>
                                    <span class="base-lvl">i{base.level_req}</span>
                                </button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>

            <!-- ── Center: config + mod picker ─────────────────────────── -->
            <div class="panel config-panel">
                {#if !selBase}
                    <p class="hint centered">Select a category, subcategory, and base item to begin crafting.</p>
                {:else if loadingBase}
                    <p class="hint">Loading mod data…</p>
                {:else}
                    <!-- Config row -->
                    <div class="config-row">
                        <label>
                            Name
                            <input class="cfg-input wide" type="text" bind:value={itemName} placeholder="Leave blank for Normal" />
                        </label>
                        <label>
                            iLvl
                            <input class="cfg-input narrow" type="number" min="1" max="100" bind:value={itemLevel} />
                        </label>
                        <label>
                            Quality
                            <input class="cfg-input narrow" type="number" min="0" max="30" bind:value={quality} />%
                        </label>
                        <label>
                            Influence
                            <select class="cfg-input" bind:value={influence} onchange={onInfluenceChange}>
                                <option value="">None</option>
                                <option value="shaper">Shaper</option>
                                <option value="elder">Elder</option>
                                <option value="crusader">Crusader</option>
                                <option value="hunter">Hunter</option>
                                <option value="redeemer">Redeemer</option>
                                <option value="warlord">Warlord</option>
                            </select>
                        </label>
                    </div>

                    <!-- Base value overrides -->
                    {#if baseProps}
                        <div class="overrides">
                            {#if baseProps.phys_damage_min != null}
                                <div class="override-row">
                                    <span class="ov-label">Phys Dmg</span>
                                    <input type="number" class="ov-input"
                                        min={baseProps.phys_damage_min} max={baseProps.phys_damage_min}
                                        bind:value={overridePhysMin} />
                                    –
                                    <input type="number" class="ov-input"
                                        min={baseProps.phys_damage_max} max={baseProps.phys_damage_max}
                                        bind:value={overridePhysMax} />
                                    <span class="ov-hint">(base: {baseProps.phys_damage_min}–{baseProps.phys_damage_max})</span>
                                </div>
                            {/if}
                            {#if baseProps.armour_max != null}
                                <div class="override-row">
                                    <span class="ov-label">Armour</span>
                                    <input type="number" class="ov-input"
                                        min={baseProps.armour_min ?? 0} max={baseProps.armour_max}
                                        bind:value={overrideArmour} />
                                    <span class="ov-hint">(max {baseProps.armour_max})</span>
                                </div>
                            {/if}
                            {#if baseProps.evasion_max != null}
                                <div class="override-row">
                                    <span class="ov-label">Evasion</span>
                                    <input type="number" class="ov-input"
                                        min={baseProps.evasion_min ?? 0} max={baseProps.evasion_max}
                                        bind:value={overrideEvasion} />
                                    <span class="ov-hint">(max {baseProps.evasion_max})</span>
                                </div>
                            {/if}
                            {#if baseProps.energy_shield_max != null}
                                <div class="override-row">
                                    <span class="ov-label">Energy Shield</span>
                                    <input type="number" class="ov-input"
                                        min={baseProps.energy_shield_min ?? 0} max={baseProps.energy_shield_max}
                                        bind:value={overrideES} />
                                    <span class="ov-hint">(max {baseProps.energy_shield_max})</span>
                                </div>
                            {/if}
                        </div>
                    {/if}

                    {#if baseModGroups}
                        <!-- Implicits -->
                        {#if baseModGroups.implicits.length > 0}
                            <div class="mod-section">
                                <h4>Implicits ({countSel(implicitSel)}/{baseModGroups.implicits.length})</h4>
                                {#each baseModGroups.implicits as group, gi}
                                    {@const sel = implicitSel[gi]}
                                    <div class="mod-row" class:active={sel != null}>
                                        <span class="mod-dn" title={group.display_name}>{group.display_name}</span>
                                        <select class="tier-sel"
                                            value={sel?.tierIdx ?? -1}
                                            onchange={(e) => {
                                                const v = parseInt((e.target as HTMLSelectElement).value);
                                                implicitSel = setTierFor(implicitSel, baseModGroups!.implicits, gi, v);
                                            }}
                                        >
                                            <option value={-1}>— None —</option>
                                            {#each group.tiers as tier, ti}
                                                <option value={ti}>{tierLabel(tier)}</option>
                                            {/each}
                                        </select>
                                    </div>
                                    {#if sel != null}
                                        {@const activeTier = group.tiers[sel.tierIdx]}
                                        {#if activeTier}
                                            <div class="mod-values">
                                                {#each activeTier.stats as stat, si}
                                                    <div class="val-row">
                                                        <span class="val-label">{stat.stat_id.replace(/_/g, '\u00a0')}</span>
                                                        {#if stat.min < stat.max}
                                                            <input type="range" class="val-slider"
                                                                min={stat.min} max={stat.max} step={1}
                                                                value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                                oninput={(e) => {
                                                                    implicitSel = setValueFor(implicitSel, gi, si, parseFloat((e.target as HTMLInputElement).value));
                                                                }}
                                                            />
                                                        {/if}
                                                        <input type="number" class="val-num"
                                                            min={stat.min} max={stat.max}
                                                            value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                            oninput={(e) => {
                                                                const v = parseFloat((e.target as HTMLInputElement).value);
                                                                if (!isNaN(v)) implicitSel = setValueFor(implicitSel, gi, si, v);
                                                            }}
                                                        />
                                                        <span class="val-range">({stat.min}–{stat.max})</span>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}
                                    {/if}
                                {/each}
                            </div>
                        {/if}

                        <!-- Prefixes -->
                        <div class="mod-section">
                            <h4>Prefixes ({countSel(prefixSel)}/3)</h4>
                            <input class="filter-input" type="text" placeholder="Filter prefixes…" bind:value={filterPrefix} />
                            <div class="mod-list">
                                {#each filteredIndices(baseModGroups.prefixes, filterPrefix) as gi}
                                    {@const group = baseModGroups.prefixes[gi]}
                                    {@const sel = prefixSel[gi]}
                                    <div class="mod-row" class:active={sel != null}>
                                        <span class="mod-dn" title={group.display_name}>{group.display_name}</span>
                                        <select class="tier-sel"
                                            value={sel?.tierIdx ?? -1}
                                            disabled={sel == null && countSel(prefixSel) >= 3}
                                            onchange={(e) => {
                                                const v = parseInt((e.target as HTMLSelectElement).value);
                                                prefixSel = setTierFor(prefixSel, baseModGroups!.prefixes, gi, v);
                                            }}
                                        >
                                            <option value={-1}>— None —</option>
                                            {#each group.tiers as tier, ti}
                                                <option value={ti}>{tierLabel(tier)}</option>
                                            {/each}
                                        </select>
                                    </div>
                                    {#if sel != null}
                                        {@const activeTier = group.tiers[sel.tierIdx]}
                                        {#if activeTier}
                                            <div class="mod-values">
                                                {#each activeTier.stats as stat, si}
                                                    <div class="val-row">
                                                        <span class="val-label">{stat.stat_id.replace(/_/g, '\u00a0')}</span>
                                                        {#if stat.min < stat.max}
                                                            <input type="range" class="val-slider"
                                                                min={stat.min} max={stat.max} step={1}
                                                                value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                                oninput={(e) => {
                                                                    prefixSel = setValueFor(prefixSel, gi, si, parseFloat((e.target as HTMLInputElement).value));
                                                                }}
                                                            />
                                                        {/if}
                                                        <input type="number" class="val-num"
                                                            min={stat.min} max={stat.max}
                                                            value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                            oninput={(e) => {
                                                                const v = parseFloat((e.target as HTMLInputElement).value);
                                                                if (!isNaN(v)) prefixSel = setValueFor(prefixSel, gi, si, v);
                                                            }}
                                                        />
                                                        <span class="val-range">({stat.min}–{stat.max})</span>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}
                                    {/if}
                                {/each}
                            </div>
                        </div>

                        <!-- Suffixes -->
                        <div class="mod-section">
                            <h4>Suffixes ({countSel(suffixSel)}/3)</h4>
                            <input class="filter-input" type="text" placeholder="Filter suffixes…" bind:value={filterSuffix} />
                            <div class="mod-list">
                                {#each filteredIndices(baseModGroups.suffixes, filterSuffix) as gi}
                                    {@const group = baseModGroups.suffixes[gi]}
                                    {@const sel = suffixSel[gi]}
                                    <div class="mod-row" class:active={sel != null}>
                                        <span class="mod-dn" title={group.display_name}>{group.display_name}</span>
                                        <select class="tier-sel"
                                            value={sel?.tierIdx ?? -1}
                                            disabled={sel == null && countSel(suffixSel) >= 3}
                                            onchange={(e) => {
                                                const v = parseInt((e.target as HTMLSelectElement).value);
                                                suffixSel = setTierFor(suffixSel, baseModGroups!.suffixes, gi, v);
                                            }}
                                        >
                                            <option value={-1}>— None —</option>
                                            {#each group.tiers as tier, ti}
                                                <option value={ti}>{tierLabel(tier)}</option>
                                            {/each}
                                        </select>
                                    </div>
                                    {#if sel != null}
                                        {@const activeTier = group.tiers[sel.tierIdx]}
                                        {#if activeTier}
                                            <div class="mod-values">
                                                {#each activeTier.stats as stat, si}
                                                    <div class="val-row">
                                                        <span class="val-label">{stat.stat_id.replace(/_/g, '\u00a0')}</span>
                                                        {#if stat.min < stat.max}
                                                            <input type="range" class="val-slider"
                                                                min={stat.min} max={stat.max} step={1}
                                                                value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                                oninput={(e) => {
                                                                    suffixSel = setValueFor(suffixSel, gi, si, parseFloat((e.target as HTMLInputElement).value));
                                                                }}
                                                            />
                                                        {/if}
                                                        <input type="number" class="val-num"
                                                            min={stat.min} max={stat.max}
                                                            value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                            oninput={(e) => {
                                                                const v = parseFloat((e.target as HTMLInputElement).value);
                                                                if (!isNaN(v)) suffixSel = setValueFor(suffixSel, gi, si, v);
                                                            }}
                                                        />
                                                        <span class="val-range">({stat.min}–{stat.max})</span>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}
                                    {/if}
                                {/each}
                            </div>
                        </div>

                        <!-- Crafted mods -->
                        <div class="mod-section">
                            <h4>Crafted ({countSel(craftedSel)}/1)</h4>
                            <input class="filter-input" type="text" placeholder="Filter crafted mods…" bind:value={filterCrafted} />
                            <div class="mod-list">
                                {#each filteredIndices(baseModGroups.crafted, filterCrafted) as gi}
                                    {@const group = baseModGroups.crafted[gi]}
                                    {@const sel = craftedSel[gi]}
                                    <div class="mod-row" class:active={sel != null}>
                                        <span class="mod-dn" title={group.display_name}>{group.display_name}</span>
                                        <select class="tier-sel"
                                            value={sel?.tierIdx ?? -1}
                                            disabled={sel == null && countSel(craftedSel) >= 1}
                                            onchange={(e) => {
                                                const v = parseInt((e.target as HTMLSelectElement).value);
                                                craftedSel = setTierFor(craftedSel, baseModGroups!.crafted, gi, v);
                                            }}
                                        >
                                            <option value={-1}>— None —</option>
                                            {#each group.tiers as tier, ti}
                                                <option value={ti}>{tierLabel(tier)}</option>
                                            {/each}
                                        </select>
                                    </div>
                                    {#if sel != null}
                                        {@const activeTier = group.tiers[sel.tierIdx]}
                                        {#if activeTier}
                                            <div class="mod-values">
                                                {#each activeTier.stats as stat, si}
                                                    <div class="val-row">
                                                        <span class="val-label">{stat.stat_id.replace(/_/g, '\u00a0')}</span>
                                                        {#if stat.min < stat.max}
                                                            <input type="range" class="val-slider"
                                                                min={stat.min} max={stat.max} step={1}
                                                                value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                                oninput={(e) => {
                                                                    craftedSel = setValueFor(craftedSel, gi, si, parseFloat((e.target as HTMLInputElement).value));
                                                                }}
                                                            />
                                                        {/if}
                                                        <input type="number" class="val-num"
                                                            min={stat.min} max={stat.max}
                                                            value={sel.values[si] ?? midpoint(stat.min, stat.max)}
                                                            oninput={(e) => {
                                                                const v = parseFloat((e.target as HTMLInputElement).value);
                                                                if (!isNaN(v)) craftedSel = setValueFor(craftedSel, gi, si, v);
                                                            }}
                                                        />
                                                        <span class="val-range">({stat.min}–{stat.max})</span>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}
                                    {/if}
                                {/each}
                            </div>
                        </div>
                    {/if}

                    <div class="action-row">
                        <button class="btn-add" disabled={!selBase || adding} onclick={addToBuild}>
                            {adding ? "Adding…" : "Add to Build Inventory"}
                        </button>
                        {#if addStatus}
                            <span class="status" class:err={addStatus.startsWith("Error")}>
                                {addStatus}
                            </span>
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- ── Right: item preview ──────────────────────────────────── -->
            <div class="panel preview-panel">
                <h3>Preview</h3>
                {#if selBase && baseModGroups}
                    {@const totalExplicit = countSel(prefixSel) + countSel(suffixSel) + countSel(craftedSel)}
                    <div class="preview-item">
                        <span
                            class="preview-name"
                            class:name-rare={totalExplicit > 2}
                            class:name-magic={totalExplicit > 0 && totalExplicit <= 2}
                        >{itemName.trim() || selBase}</span>
                        <div class="preview-base">
                            {selBase}
                            {#if influence}<span class="preview-inf"> · {influence.charAt(0).toUpperCase() + influence.slice(1)}</span>{/if}
                        </div>
                        {#if baseProps}
                            {#if baseProps.armour_max != null}
                                <div class="preview-stat">Armour: <strong>{(overrideArmour ?? baseProps.armour_max)?.toFixed(0)}</strong></div>
                            {/if}
                            {#if baseProps.evasion_max != null}
                                <div class="preview-stat">Evasion: <strong>{(overrideEvasion ?? baseProps.evasion_max)?.toFixed(0)}</strong></div>
                            {/if}
                            {#if baseProps.energy_shield_max != null}
                                <div class="preview-stat">Energy Shield: <strong>{(overrideES ?? baseProps.energy_shield_max)?.toFixed(0)}</strong></div>
                            {/if}
                            {#if baseProps.phys_damage_min != null}
                                <div class="preview-stat">
                                    Phys: <strong>{overridePhysMin ?? baseProps.phys_damage_min}–{overridePhysMax ?? baseProps.phys_damage_max}</strong>
                                    {#if baseProps.attack_time_ms}<span class="gray"> @ {(1000 / baseProps.attack_time_ms).toFixed(2)} APS</span>{/if}
                                </div>
                            {/if}
                        {/if}

                        {#if countSel(implicitSel) > 0}
                            <hr class="pdiv" />
                            {#each Object.entries(implicitSel) as [idxStr, s]}
                                {@const group = baseModGroups.implicits[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod implicit">{fillTemplate(group.display_name, s.values)}</div>
                                {/if}
                            {/each}
                        {/if}

                        {#if totalExplicit > 0}
                            <hr class="pdiv" />
                            {#each Object.entries(prefixSel) as [idxStr, s]}
                                {@const group = baseModGroups.prefixes[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod">{fillTemplate(group.display_name, s.values)}</div>
                                {/if}
                            {/each}
                            {#each Object.entries(suffixSel) as [idxStr, s]}
                                {@const group = baseModGroups.suffixes[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod">{fillTemplate(group.display_name, s.values)}</div>
                                {/if}
                            {/each}
                            {#each Object.entries(craftedSel) as [idxStr, s]}
                                {@const group = baseModGroups.crafted[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod crafted">{fillTemplate(group.display_name, s.values)}</div>
                                {/if}
                            {/each}
                        {/if}
                    </div>
                {:else if selBase}
                    <p class="hint">Loading…</p>
                {:else}
                    <p class="hint">No item selected.</p>
                {/if}
            </div>

        </section>
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
        background-color: #0e0e10;
        color: #e0d6c2;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    }

    .craft-page { display: flex; min-height: 100vh; }
    .page-body { flex: 1; min-width: 0; display: flex; flex-direction: column; }
    .content { display: flex; gap: 10px; padding: 10px 14px; flex: 1; overflow: hidden; min-height: 0; }

    .panel {
        background: #111115;
        border: 1px solid #2a2a30;
        border-radius: 6px;
        padding: 10px;
        overflow-y: auto;
    }

    h3 {
        margin: 0 0 10px;
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.07em;
        color: #c8a96e;
    }
    h4 {
        margin: 10px 0 5px;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: #9b8a5e;
        border-bottom: 1px solid #2a2a30;
        padding-bottom: 3px;
    }

    /* ── Browser panel ─────────────────────────────────────────────────────── */
    .browser-panel {
        flex: 0 0 200px;
        display: flex;
        flex-direction: column;
        gap: 0;
        padding: 8px;
    }

    .cat-tabs {
        display: flex;
        flex-wrap: wrap;
        gap: 3px;
        margin-bottom: 8px;
    }
    .cat-tab {
        flex: 1 1 auto;
        background: #1a1a20;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #888;
        cursor: pointer;
        font-size: 0.68rem;
        padding: 4px 5px;
        text-align: center;
        transition: background 0.12s, color 0.12s;
    }
    .cat-tab:hover { background: #22222a; color: #c8b080; }
    .cat-tab.active { background: #22201a; border-color: #c8a96e; color: #c8a96e; }

    .subcat-list {
        display: flex;
        flex-direction: column;
        gap: 1px;
        max-height: 180px;
        overflow-y: auto;
        margin-bottom: 4px;
    }
    .subcat-btn {
        background: transparent;
        border: none;
        border-radius: 3px;
        color: #888;
        cursor: pointer;
        font-size: 0.75rem;
        padding: 4px 7px;
        text-align: left;
        transition: background 0.1s, color 0.1s;
    }
    .subcat-btn:hover { background: #1e1e26; color: #c8b080; }
    .subcat-btn.active { background: #22201a; color: #c8a96e; font-weight: 500; }

    .base-sep {
        height: 1px;
        background: #2a2a30;
        margin: 4px 0;
    }

    .base-list {
        list-style: none;
        margin: 0;
        padding: 0;
        flex: 1;
        overflow-y: auto;
    }
    .base-item { border-bottom: 1px solid #1a1a20; }
    .base-item.active .base-btn { background: #22201a; border-left: 2px solid #c8a96e; }
    .base-btn {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        background: transparent;
        border: none;
        color: inherit;
        cursor: pointer;
        font-size: 0.74rem;
        padding: 4px 7px;
        text-align: left;
    }
    .base-btn:hover { background: #1e1e26; }
    .base-name { color: #d0c8b0; }
    .base-lvl { color: #555; font-size: 0.65rem; }

    /* ── Config panel ──────────────────────────────────────────────────────── */
    .config-panel {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
    }

    .config-row {
        display: flex;
        gap: 10px;
        align-items: flex-end;
        margin-bottom: 8px;
        flex-wrap: wrap;
    }
    .config-row label {
        display: flex;
        flex-direction: column;
        gap: 3px;
        font-size: 0.72rem;
        color: #888;
    }
    .cfg-input {
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.8rem;
        padding: 4px 6px;
    }
    .cfg-input.wide   { width: 150px; }
    .cfg-input.narrow { width: 55px; }

    /* Overrides */
    .overrides { margin-bottom: 8px; }
    .override-row {
        display: flex;
        align-items: center;
        gap: 5px;
        font-size: 0.73rem;
        margin-bottom: 3px;
        color: #888;
    }
    .ov-label { min-width: 80px; color: #888; }
    .ov-input {
        width: 58px;
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.75rem;
        padding: 2px 4px;
    }
    .ov-hint { color: #484840; font-size: 0.68rem; }

    /* Mod sections */
    .mod-section { margin-bottom: 8px; }

    .filter-input {
        width: 100%;
        box-sizing: border-box;
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.75rem;
        padding: 4px 8px;
        margin-bottom: 3px;
        outline: none;
    }
    .filter-input:focus { border-color: #5a5a60; }

    .mod-list {
        max-height: 260px;
        overflow-y: auto;
        border: 1px solid #1e1e24;
        border-radius: 3px;
        background: #0e0e12;
    }

    .mod-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 6px;
        padding: 4px 7px;
        border-bottom: 1px solid #181820;
        min-height: 28px;
    }
    .mod-row.active {
        background: #141420;
        border-left: 2px solid #4a6090;
    }
    .mod-row:last-child { border-bottom: none; }

    .mod-dn {
        flex: 1;
        font-size: 0.74rem;
        color: #b0a888;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .mod-row.active .mod-dn { color: #c8b888; }

    .tier-sel {
        flex: 0 0 auto;
        max-width: 160px;
        background: #1a1a22;
        border: 1px solid #3a3a48;
        border-radius: 3px;
        color: #9eb0c8;
        font-size: 0.7rem;
        padding: 2px 4px;
        cursor: pointer;
    }
    .tier-sel:disabled { opacity: 0.35; cursor: not-allowed; }
    .tier-sel:focus { border-color: #5a6a80; outline: none; }

    /* Value controls */
    .mod-values {
        padding: 4px 7px 6px 14px;
        background: #0c0c18;
        border-bottom: 1px solid #181820;
    }
    .val-row {
        display: flex;
        align-items: center;
        gap: 5px;
        margin-top: 3px;
    }
    .val-label {
        font-size: 0.66rem;
        color: #666;
        min-width: 100px;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .val-slider {
        flex: 1;
        max-width: 100px;
        accent-color: #6a90c8;
        cursor: pointer;
        height: 3px;
    }
    .val-num {
        width: 52px;
        background: #18181e;
        border: 1px solid #3a3a48;
        border-radius: 3px;
        color: #9eb0c8;
        font-size: 0.72rem;
        padding: 2px 4px;
        text-align: right;
    }
    .val-range {
        font-size: 0.62rem;
        color: #444;
        white-space: nowrap;
    }

    /* Action row */
    .action-row {
        margin-top: 12px;
        display: flex;
        align-items: center;
        gap: 12px;
    }
    .btn-add {
        background: #3a2e14;
        border: 1px solid #c8a96e;
        border-radius: 4px;
        color: #c8a96e;
        cursor: pointer;
        font-size: 0.82rem;
        padding: 7px 16px;
        transition: background 0.15s;
    }
    .btn-add:hover:not(:disabled) { background: #4a3e1e; }
    .btn-add:disabled { opacity: 0.4; cursor: not-allowed; }
    .status { font-size: 0.78rem; color: #9eb9a4; }
    .status.err { color: #c87070; }

    /* ── Preview panel ─────────────────────────────────────────────────────── */
    .preview-panel { flex: 0 0 220px; font-size: 0.78rem; }

    .preview-item { display: flex; flex-direction: column; gap: 2px; }

    .preview-name { font-size: 0.88rem; font-weight: 600; color: #e0d6c2; }
    .preview-name.name-magic { color: #8888ff; }
    .preview-name.name-rare  { color: #ffd700; }

    .preview-base { color: #888; font-size: 0.72rem; margin-bottom: 3px; }
    .preview-inf  { color: #6a9aca; }

    .preview-stat { color: #c8b88a; }
    .preview-stat .gray { color: #666; }

    .pdiv { border: none; border-top: 1px solid #2e2c24; margin: 5px 0; }

    .preview-mod { color: #d0c0a0; }
    .preview-mod.implicit { color: #88aacc; }
    .preview-mod.crafted  { color: #a8c8f8; }

    .hint { color: #555; font-size: 0.8rem; margin: 8px 0; }
    .hint.centered { text-align: center; margin-top: 40px; }
</style>
