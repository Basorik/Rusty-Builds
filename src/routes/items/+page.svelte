<script lang="ts">
    import Header from "../../components/Header.svelte";
    import Sidebar from "../../components/Sidebar.svelte";
    import { getBuildState } from "$lib/buildState.svelte";
    import { commands } from "../../bindings";
    import type {
        ItemSlot,
        UniqueSearchResult,
        InventoryItemSummary,
        BaseCategory,
        BaseItemProps,
        BaseModGroups,
        CraftedItemSpec,
        CraftedModValue,
        ModTierInfo,
        StatModGroup,
        UniqueListItem,
        UniqueDetail,
        ItemDetail,
    } from "../../bindings";

    const build = getBuildState();

    // ─── Inventory / equip ───────────────────────────────────────────────────
    let slotInventoryItems = $state<InventoryItemSummary[]>([]);
    let loadingInventory = $state(false);
    /** Detail for the currently hovered/clicked inventory item */
    let hoveredInvDetail = $state<ItemDetail | null>(null);
    let hoveredInvId = $state<number | null>(null);

    async function loadInvDetail(id: number) {
        if (hoveredInvId === id) return;
        hoveredInvId = id;
        const res = await commands.getItemDetailById(id);
        if (res.status === "ok") hoveredInvDetail = res.data;
    }
    function clearInvHover() {
        hoveredInvId = null;
        hoveredInvDetail = null;
    }

    // ─── Unique browser (Craft → Unique category) ────────────────────────────
    let uniqueClassFilter = $state("All");
    let uniqueQuery = $state("");
    let uniqueList = $state<UniqueListItem[]>([]);
    let uniqueSearching = $state(false);
    let selUnique = $state<UniqueDetail | null>(null);
    /** Flat array of chosen roll values ordered as: implicit_lines then explicit_lines,
     *  then by ranges within each line, left-to-right. */
    let uniqueRolls = $state<number[]>([]);
    let addUniqueStatus = $state("");
    let addingUnique = $state(false);

    // ─── Craft: navigation ───────────────────────────────────────────────────
    let categories = $state<BaseCategory[]>([]);
    let categoryLoaded = $state(false);
    let selCatName = $state("");
    let selSubLabel = $state("");
    let selBase = $state("");

    let selCategory = $derived(
        categories.find((c) => c.name === selCatName) ?? null,
    );
    let selSubcategory = $derived(
        selCategory?.subcategories.find((s) => s.label === selSubLabel) ?? null,
    );

    // ─── Craft: base data ────────────────────────────────────────────────────
    let baseProps = $state<BaseItemProps | null>(null);
    let baseModGroups = $state<BaseModGroups | null>(null);
    let loadingBase = $state(false);

    // ─── Craft: config ───────────────────────────────────────────────────────
    let itemName = $state("");
    let itemLevel = $state(100);
    let quality = $state(20);
    let influence = $state("");

    let overridePhysMin = $state<number | null>(null);
    let overridePhysMax = $state<number | null>(null);
    let overrideArmour = $state<number | null>(null);
    let overrideEvasion = $state<number | null>(null);
    let overrideES = $state<number | null>(null);

    // ─── Craft: mod selections ───────────────────────────────────────────────
    type SelMap = Record<number, { tierIdx: number; values: number[] }>;
    let implicitSel = $state<SelMap>({});
    let prefixSel = $state<SelMap>({});
    let suffixSel = $state<SelMap>({});
    let craftedSel = $state<SelMap>({});

    let filterPrefix = $state("");
    let filterSuffix = $state("");
    let filterCrafted = $state("");

    let addStatus = $state("");
    let adding = $state(false);

    // Tooltip state (equipped items only)
    type TooltipData = {
        kind: "equipped";
        slot: string;
        name: string;
        base_name: string;
        item_class: string;
        total_dps: number | null;
        armour: number | null;
        evasion: number | null;
        energy_shield: number | null;
        mod_count: number;
    };
    let tooltipData = $state<TooltipData | null>(null);
    let tooltipX = $state(0);
    let tooltipY = $state(0);

    function showTooltip(e: MouseEvent, data: TooltipData) {
        tooltipData = data;
        tooltipX = e.clientX + 14;
        tooltipY = e.clientY + 14;
    }
    function moveTooltip(e: MouseEvent) {
        tooltipX = e.clientX + 14;
        tooltipY = e.clientY + 14;
    }
    function hideTooltip() {
        tooltipData = null;
    }

    async function doSearch() {
        // legacy stub — search is now in the Craft → Unique browser
    }

    async function equipSelected() {
        // legacy stub — add to inventory then equip from Inventory tab
    }

    async function loadSlotInventory(slot: ItemSlot) {
        loadingInventory = true;
        const res = await commands.getInventoryForSlot(slot);
        if (res.status === "ok") slotInventoryItems = res.data;
        loadingInventory = false;
    }

    async function selectSlot(slot: ItemSlot) {
        build.selectedEquipSlot = slot;
        slotInventoryItems = [];
        await loadSlotInventory(slot);
    }

    async function equipFromInventory(inventoryId: number, itemClass: string) {
        const validSlots = CLASS_TO_SLOTS[itemClass] ?? [];
        if (validSlots.length === 0) return;

        // Use the selected slot if it's compatible; otherwise auto-pick the first free slot.
        let targetSlot: string;
        if (
            build.selectedEquipSlot &&
            validSlots.includes(build.selectedEquipSlot)
        ) {
            targetSlot = build.selectedEquipSlot;
        } else {
            const occupied = new Set(build.equippedItems.map((e) => e.slot));
            targetSlot =
                validSlots.find((s) => !occupied.has(s)) ?? validSlots[0];
        }

        const res = await commands.equipFromInventory(
            inventoryId,
            targetSlot as ItemSlot,
        );
        if (res.status === "ok") {
            build.buildStats = res.data;
            await refreshEquipped();
            await refreshInventory();
            if (build.selectedEquipSlot)
                await loadSlotInventory(build.selectedEquipSlot);
        }
    }

    async function removeFromInventory(inventoryId: number) {
        const res = await commands.removeInventoryItem(inventoryId);
        if (res.status === "ok") {
            build.buildStats = res.data;
            await refreshInventory();
            if (build.selectedEquipSlot)
                await loadSlotInventory(build.selectedEquipSlot);
        }
    }

    async function unequip(slot: ItemSlot) {
        const res = await commands.unequipItem(slot);
        if (res.status === "ok") {
            build.buildStats = res.data;
            await refreshEquipped();
            await refreshInventory();
            if (build.selectedEquipSlot)
                await loadSlotInventory(build.selectedEquipSlot);
        }
    }

    async function refreshEquipped() {
        const res = await commands.getEquippedItems();
        if (res.status === "ok") build.equippedItems = res.data;
    }

    async function refreshInventory() {
        const res = await commands.getInventoryItems();
        if (res.status === "ok") build.inventoryItems = res.data;
    }

    // Rarity colour for inventory items
    function rarityColour(rarity: string): string {
        switch (rarity) {
            case "Normal":
                return "#e0d6c2";
            case "Magic":
                return "#8888ff";
            case "Rare":
                return "#ffd700";
            case "Unique":
                return "#c8a96e";
            default:
                return "#e0d6c2";
        }
    }

    // ─── Craft: navigation functions ─────────────────────────────────────────
    async function loadCraftCategories() {
        if (!categoryLoaded) {
            const res = await commands.getBaseCategories();
            if (res.status === "ok") {
                categories = res.data;
                categoryLoaded = true;
            }
        }
    }

    // Load craft categories on mount
    $effect(() => {
        loadCraftCategories();
    });

    function selectCategory(name: string) {
        selCatName = name;
        selSubLabel = "";
        selBase = "";
        baseProps = null;
        baseModGroups = null;
        clearSelections();
        // Clear unique browser state when switching away
        selUnique = null;
        uniqueList = [];
        uniqueRolls = [];
        uniqueQuery = "";
        uniqueClassFilter = "All";
        addUniqueStatus = "";
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
            overrideArmour = p.armour_max;
            overrideEvasion = p.evasion_max;
            overrideES = p.energy_shield_max;
        }
        if (modsRes.status === "ok") {
            baseModGroups = modsRes.data;
            const newSel: SelMap = {};
            modsRes.data.implicits.forEach((group, i) => {
                if (group.tiers.length > 0) {
                    const t = group.tiers[0];
                    newSel[i] = {
                        tierIdx: 0,
                        values: t.stats.map((s) => midpoint(s.min, s.max)),
                    };
                }
            });
            implicitSel = newSel;
        }
    }

    function clearSelections() {
        implicitSel = {};
        prefixSel = {};
        suffixSel = {};
        craftedSel = {};
        overridePhysMin = null;
        overridePhysMax = null;
        overrideArmour = null;
        overrideEvasion = null;
        overrideES = null;
    }

    // ─── Unique browser functions ─────────────────────────────────────────────

    /** Computed flat roll-offset for each (implicit + explicit) line in selUnique. */
    let uniqueLineOffsets = $derived.by((): number[] => {
        if (!selUnique) return [];
        const all = [...selUnique.implicit_lines, ...selUnique.explicit_lines];
        const offsets: number[] = [];
        let cur = 0;
        for (const line of all) {
            offsets.push(cur);
            cur += line.ranges.length;
        }
        return offsets;
    });

    /** Replace (X-Y) placeholders in a line's text with the current roll values. */
    function applyRollsToText(text: string, lineIdx: number): string {
        let ri = 0;
        const base = uniqueLineOffsets[lineIdx] ?? 0;
        return text.replace(
            /\(-?[0-9]+(?:\.[0-9]+)?--?[0-9]+(?:\.[0-9]+)?\)/g,
            () => {
                const v = uniqueRolls[base + ri++] ?? 0;
                return v === Math.trunc(v) ? v.toFixed(0) : v.toFixed(2);
            },
        );
    }

    async function loadUniquesForClass() {
        uniqueSearching = true;
        selUnique = null;
        uniqueRolls = [];
        addUniqueStatus = "";
        const cls = uniqueClassFilter !== "All" ? uniqueClassFilter : null;
        const res = await commands.getUniquesForClass(cls, uniqueQuery);
        if (res.status === "ok") uniqueList = res.data;
        uniqueSearching = false;
    }

    async function onSelectUnique(name: string) {
        const res = await commands.getUniqueDetail(name);
        if (res.status === "ok") {
            selUnique = res.data;
            addUniqueStatus = "";
            // Initialise rolls to the midpoint of each range.
            const all = [
                ...res.data.implicit_lines,
                ...res.data.explicit_lines,
            ];
            uniqueRolls = all.flatMap((line) =>
                line.ranges.map(([lo, hi]) => {
                    const mid = (lo + hi) / 2;
                    return lo === hi
                        ? lo
                        : Number.isInteger(lo) && Number.isInteger(hi)
                          ? Math.round(mid)
                          : mid;
                }),
            );
        }
    }

    async function addUniqueToBuild() {
        if (!selUnique) return;
        addingUnique = true;
        addUniqueStatus = "";
        const res = await commands.addUniqueToInventory(
            selUnique.name,
            uniqueRolls,
        );
        if (res.status === "ok") {
            build.buildStats = res.data;
            await refreshInventory();
            if (build.selectedEquipSlot)
                await loadSlotInventory(build.selectedEquipSlot);
            addUniqueStatus = `"${selUnique.name}" added to inventory.`;
        } else {
            addUniqueStatus = `Error: ${res.error}`;
        }
        addingUnique = false;
    }

    // ─── Craft: mod picking helpers ──────────────────────────────────────────
    function midpoint(min: number, max: number): number {
        return min === max ? min : Math.round((min + max) / 2);
    }

    function setTierFor(
        sel: SelMap,
        groups: StatModGroup[],
        gi: number,
        tierIdx: number,
    ): SelMap {
        if (tierIdx < 0) {
            const next = { ...sel };
            delete next[gi];
            return next;
        }
        const tier = groups[gi].tiers[tierIdx];
        return {
            ...sel,
            [gi]: {
                tierIdx,
                values: tier.stats.map((s) => midpoint(s.min, s.max)),
            },
        };
    }

    function setValueFor(
        sel: SelMap,
        gi: number,
        si: number,
        value: number,
    ): SelMap {
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
        if (tier.stats.length === 0)
            return `T${tier.tier} (lv${tier.required_level})`;
        const s = tier.stats[0];
        const range =
            s.min === s.max
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
                g.tiers.some((t) =>
                    t.stats.some((s) => s.stat_id.toLowerCase().includes(q)),
                )
            ) {
                acc.push(i);
            }
            return acc;
        }, []);
    }

    function buildModValues(
        sel: SelMap,
        groups: StatModGroup[],
    ): CraftedModValue[] {
        return Object.entries(sel).map(([idxStr, s]) => ({
            mod_id: groups[parseInt(idxStr)].tiers[s.tierIdx].mod_id,
            values: s.values,
        }));
    }

    function fillTemplate(name: string, values: number[]): string {
        let i = 0;
        return name.replace(/#/g, () => String(Math.round(values[i++] ?? 0)));
    }

    const INFLUENCE_FLAGS: [number, string][] = [
        [1 << 0, "Shaper"],
        [1 << 1, "Elder"],
        [1 << 2, "Crusader"],
        [1 << 3, "Hunter"],
        [1 << 4, "Redeemer"],
        [1 << 5, "Warlord"],
    ];

    function influenceList(bits: number): string[] {
        return INFLUENCE_FLAGS.filter(([bit]) => bits & bit).map(
            ([, name]) => name,
        );
    }

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
            base_armour: overrideArmour,
            base_evasion: overrideEvasion,
            base_energy_shield: overrideES,
            implicits: buildModValues(implicitSel, baseModGroups.implicits),
            prefixes: buildModValues(prefixSel, baseModGroups.prefixes),
            suffixes: buildModValues(suffixSel, baseModGroups.suffixes),
            crafted: buildModValues(craftedSel, baseModGroups.crafted),
            influence: influence || null,
        };
        const res = await commands.addCraftedItem(spec);
        if (res.status === "ok") {
            build.buildStats = res.data;
            await refreshInventory();
            if (build.selectedEquipSlot)
                await loadSlotInventory(build.selectedEquipSlot);
            addStatus = `"${itemName.trim() || selBase}" added to inventory.`;
        } else {
            addStatus = `Error: ${res.error}`;
        }
        adding = false;
    }

    // Equipment grid layout — PoE standard layout
    const SLOT_GRID: Array<{
        slot: ItemSlot;
        label: string;
        col: number;
        row: number;
    }> = [
        { slot: "Helmet", label: "Helm", col: 2, row: 1 },
        { slot: "Amulet", label: "Amulet", col: 3, row: 1 },
        { slot: "Weapon1", label: "Weap", col: 1, row: 2 },
        { slot: "BodyArmour", label: "Body", col: 2, row: 2 },
        { slot: "Weapon2", label: "Off", col: 3, row: 2 },
        { slot: "Ring1", label: "Ring", col: 1, row: 3 },
        { slot: "Gloves", label: "Gloves", col: 2, row: 3 },
        { slot: "Ring2", label: "Ring", col: 3, row: 3 },
        { slot: "Belt", label: "Belt", col: 2, row: 4 },
        { slot: "Boots", label: "Boots", col: 2, row: 5 },
        { slot: "Flask1", label: "F1", col: 1, row: 6 },
        { slot: "Flask2", label: "F2", col: 1, row: 6 },
        { slot: "Flask3", label: "F3", col: 2, row: 6 },
        { slot: "Flask4", label: "F4", col: 3, row: 6 },
        { slot: "Flask5", label: "F5", col: 3, row: 6 },
    ];

    function equippedInSlot(slot: ItemSlot) {
        return build.equippedItems.find((e) => e.slot === slot) ?? null;
    }

    // Map item_class strings (as stored by the backend) to the slot names they can go into.
    // Keys must match EQUIPPABLE_CLASSES in src-tauri/src/data/bases.rs exactly.
    const CLASS_TO_SLOTS: Record<string, string[]> = {
        "One Hand Sword": ["Weapon1", "Weapon2"],
        "Thrusting One Hand Sword": ["Weapon1", "Weapon2"],
        "Two Hand Sword": ["Weapon1"],
        "One Hand Axe": ["Weapon1", "Weapon2"],
        "Two Hand Axe": ["Weapon1"],
        "One Hand Mace": ["Weapon1", "Weapon2"],
        "Two Hand Mace": ["Weapon1"],
        Bow: ["Weapon1"],
        Staff: ["Weapon1"],
        Warstaff: ["Weapon1"],
        Wand: ["Weapon1", "Weapon2"],
        Dagger: ["Weapon1", "Weapon2"],
        "Rune Dagger": ["Weapon1", "Weapon2"],
        Claw: ["Weapon1", "Weapon2"],
        Sceptre: ["Weapon1", "Weapon2"],
        Shield: ["Weapon2"],
        Quiver: ["Weapon2"],
        Helmet: ["Helmet"],
        "Body Armour": ["BodyArmour"],
        Gloves: ["Gloves"],
        Boots: ["Boots"],
        Amulet: ["Amulet"],
        Ring: ["Ring1", "Ring2"],
        Belt: ["Belt"],
        LifeFlask: ["Flask1", "Flask2", "Flask3", "Flask4", "Flask5"],
        ManaFlask: ["Flask1", "Flask2", "Flask3", "Flask4", "Flask5"],
        HybridFlask: ["Flask1", "Flask2", "Flask3", "Flask4", "Flask5"],
        UtilityFlask: ["Flask1", "Flask2", "Flask3", "Flask4", "Flask5"],
    };

    /** Item classes for the unique browser class-filter dropdown. */
    const ALL_ITEM_CLASSES = ["All", ...Object.keys(CLASS_TO_SLOTS).sort()];

    /** Order in which item-class groups appear in the inventory list */
    const INV_CLASS_ORDER: string[] = [
        "One Hand Sword",
        "Thrusting One Hand Sword",
        "One Hand Axe",
        "One Hand Mace",
        "Two Hand Sword",
        "Two Hand Axe",
        "Two Hand Mace",
        "Staff",
        "Warstaff",
        "Bow",
        "Wand",
        "Dagger",
        "Rune Dagger",
        "Claw",
        "Sceptre",
        "Shield",
        "Quiver",
        "Helmet",
        "Body Armour",
        "Gloves",
        "Boots",
        "Amulet",
        "Ring",
        "Belt",
        "LifeFlask",
        "ManaFlask",
        "HybridFlask",
        "UtilityFlask",
    ];

    /** All inventory items grouped by item_class in slot order */
    let groupedInventory = $derived.by(() => {
        const all = build.inventoryItems;
        const map = new Map<string, InventoryItemSummary[]>();
        for (const item of all) {
            const cls = item.item_class || "Other";
            if (!map.has(cls)) map.set(cls, []);
            map.get(cls)!.push(item);
        }
        const ordered: Array<{ cls: string; items: InventoryItemSummary[] }> =
            [];
        for (const cls of INV_CLASS_ORDER) {
            if (map.has(cls)) {
                ordered.push({ cls, items: map.get(cls)! });
                map.delete(cls);
            }
        }
        // Any remaining classes not in the order list
        for (const [cls, items] of map) {
            ordered.push({ cls, items });
        }
        return ordered;
    });

    /** Check if an unequipped item has at least one valid equip slot */
    function hasAnyValidSlot(item: InventoryItemSummary): boolean {
        return (CLASS_TO_SLOTS[item.item_class] ?? []).length > 0;
    }

    // Preview: item to show in the always-visible preview pane
    let previewEquippedItem = $derived(
        build.selectedEquipSlot
            ? equippedInSlot(build.selectedEquipSlot)
            : null,
    );
    // Full PoE-style detail for the equipped preview — loaded via IPC when slot/item changes
    let previewItemDetail = $state<ItemDetail | null>(null);
    $effect(() => {
        // Re-fetch whenever the selected slot or its equipped item changes
        const slot = build.selectedEquipSlot;
        const item = slot ? equippedInSlot(slot) : null;
        if (slot && item) {
            commands.getItemDetailBySlot(slot).then((res) => {
                if (res.status === "ok") previewItemDetail = res.data;
                else previewItemDetail = null;
            });
        } else {
            previewItemDetail = null;
        }
    });

    /** The detail to show in the preview panel: hovered inv item takes priority */
    let activePreviewDetail = $derived(hoveredInvDetail ?? previewItemDetail);
</script>

<main class="items-page">
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
            <!-- Left: equipment grid -->
            <div class="equip-panel">
                <h3>Equipment</h3>
                <div class="equip-grid">
                    {#each SLOT_GRID as cell}
                        {@const item = equippedInSlot(cell.slot)}
                        <button
                            class="slot-btn"
                            class:selected={build.selectedEquipSlot ===
                                cell.slot}
                            class:occupied={item !== null}
                            style="grid-column:{cell.col};grid-row:{cell.row}"
                            onclick={() => selectSlot(cell.slot)}
                            onmouseenter={(e) =>
                                item &&
                                showTooltip(e, {
                                    kind: "equipped",
                                    slot: cell.slot,
                                    name: item.name,
                                    base_name: item.base_name,
                                    item_class: item.item_class,
                                    total_dps: item.total_dps,
                                    armour: item.armour,
                                    evasion: item.evasion,
                                    energy_shield: item.energy_shield,
                                    mod_count: item.mod_count,
                                })}
                            onmousemove={moveTooltip}
                            onmouseleave={hideTooltip}
                        >
                            {#if item}
                                <span class="slot-name">{item.name}</span>
                                {#if item.total_dps != null}
                                    <span class="slot-stat"
                                        >{item.total_dps.toFixed(0)} dps</span
                                    >
                                {:else if item.armour != null}
                                    <span class="slot-stat"
                                        >{item.armour.toFixed(0)} ar</span
                                    >
                                {:else if item.energy_shield != null}
                                    <span class="slot-stat"
                                        >{item.energy_shield.toFixed(0)} es</span
                                    >
                                {/if}
                            {:else}
                                <span class="slot-label">{cell.label}</span>
                            {/if}
                        </button>
                    {/each}
                </div>
                {#if build.selectedEquipSlot}
                    {@const item = equippedInSlot(build.selectedEquipSlot)}
                    {#if item}
                        <div class="slot-actions">
                            <span class="equipped-label">{item.name}</span>
                            <button
                                class="btn-danger"
                                onclick={() =>
                                    unequip(build.selectedEquipSlot!)}
                            >
                                Unequip
                            </button>
                        </div>
                    {:else}
                        <div class="slot-actions">
                            <span class="empty-label"
                                >Slot: {build.selectedEquipSlot}</span
                            >
                        </div>
                    {/if}
                {/if}
            </div>

            <!-- Center: inventory panel (always visible) -->
            <div class="inv-panel-outer">
                <h3>Inventory ({build.inventoryItems.length})</h3>
                <div class="inv-panel">
                    {#if build.inventoryItems.length === 0}
                        <p class="hint">
                            No items in inventory yet.<br />Use the Craft panel
                            to add items.
                        </p>
                    {:else}
                        {#each groupedInventory as group}
                            <div class="inv-group">
                                <div class="inv-group-header">{group.cls}</div>
                                {#each group.items as item}
                                    {@const isEquipped =
                                        item.equipped_slot !== null}
                                    {@const compatible =
                                        !isEquipped && hasAnyValidSlot(item)}
                                    <div
                                        class="inv-item"
                                        class:inv-item-equipped={isEquipped}
                                        class:inv-item-compatible={compatible}
                                        role="listitem"
                                        onmouseenter={() =>
                                            loadInvDetail(item.inventory_id)}
                                        onmouseleave={clearInvHover}
                                    >
                                        <div class="inv-item-info">
                                            <div class="inv-name-row">
                                                <span
                                                    class="inv-name"
                                                    style="color:{rarityColour(
                                                        item.rarity,
                                                    )}"
                                                    >{item.name ||
                                                        item.base_name}</span
                                                >
                                                {#if isEquipped}
                                                    <span
                                                        class="inv-equipped-badge"
                                                        >{item.equipped_slot}</span
                                                    >
                                                {/if}
                                            </div>
                                            <span class="inv-base"
                                                >{item.base_name}</span
                                            >
                                            <div class="inv-stats">
                                                {#if item.total_dps != null}
                                                    <span
                                                        >{item.total_dps.toFixed(
                                                            0,
                                                        )} dps</span
                                                    >
                                                {/if}
                                                {#if item.armour != null}
                                                    <span
                                                        >{item.armour.toFixed(
                                                            0,
                                                        )} ar</span
                                                    >
                                                {/if}
                                                {#if item.evasion != null}
                                                    <span
                                                        >{item.evasion.toFixed(
                                                            0,
                                                        )} ev</span
                                                    >
                                                {/if}
                                                {#if item.energy_shield != null}
                                                    <span
                                                        >{item.energy_shield.toFixed(
                                                            0,
                                                        )} es</span
                                                    >
                                                {/if}
                                                <span class="inv-mods"
                                                    >{item.mod_count} mods</span
                                                >
                                            </div>
                                        </div>
                                        <div class="inv-actions">
                                            {#if isEquipped}
                                                <button
                                                    class="btn-remove-inv"
                                                    title="Unequip"
                                                    onclick={() =>
                                                        unequip(
                                                            item.equipped_slot!,
                                                        )}
                                                >
                                                    ↩
                                                </button>
                                            {:else}
                                                <button
                                                    class="btn-equip-inv"
                                                    disabled={!compatible}
                                                    title={compatible
                                                        ? "Equip"
                                                        : "No valid slot for this item"}
                                                    onclick={() =>
                                                        equipFromInventory(
                                                            item.inventory_id,
                                                            item.item_class,
                                                        )}>Equip</button
                                                >
                                            {/if}
                                            <button
                                                class="btn-remove-inv"
                                                onclick={() =>
                                                    removeFromInventory(
                                                        item.inventory_id,
                                                    )}>✕</button
                                            >
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>

            <!-- Right: craft panel (always visible) -->
            <div class="craft-panel-outer">
                <div class="craft-inner">
                    <!-- Left: hierarchical base browser -->
                    <div class="craft-browser">
                        <div class="cat-tabs">
                            {#each categories as cat}
                                <button
                                    class="cat-tab"
                                    class:active={selCatName === cat.name}
                                    onclick={() => selectCategory(cat.name)}
                                    >{cat.name}</button
                                >
                            {/each}
                        </div>
                        {#if selCatName === "Unique"}
                            <!-- Unique browser: class filter + name search + list -->
                            <div class="uniq-filters">
                                <select
                                    class="uniq-class-sel"
                                    bind:value={uniqueClassFilter}
                                    onchange={loadUniquesForClass}
                                >
                                    {#each ALL_ITEM_CLASSES as cls}
                                        <option value={cls}>{cls}</option>
                                    {/each}
                                </select>
                                <input
                                    class="uniq-search"
                                    type="text"
                                    placeholder="Name…"
                                    bind:value={uniqueQuery}
                                    oninput={loadUniquesForClass}
                                />
                            </div>
                            {#if uniqueSearching}
                                <p class="hint">Loading…</p>
                            {:else if uniqueList.length === 0 && (uniqueQuery.trim() || uniqueClassFilter !== "All")}
                                <p class="hint">No uniques found.</p>
                            {:else if uniqueList.length === 0}
                                <p class="hint" style="font-size:0.7rem">
                                    Type a name or select an item class to
                                    browse uniques.
                                </p>
                            {:else}
                                <ul class="base-list">
                                    {#each uniqueList as u}
                                        <li
                                            class="base-item"
                                            class:active={selUnique?.name ===
                                                u.name}
                                        >
                                            <button
                                                class="base-btn uniq-list-btn"
                                                onclick={() =>
                                                    onSelectUnique(u.name)}
                                            >
                                                <div class="uniq-list-namecol">
                                                    <span class="uniq-list-name"
                                                        >{u.name}</span
                                                    >
                                                    <span class="uniq-list-base"
                                                        >{u.base_name}</span
                                                    >
                                                </div>
                                                {#if u.league}
                                                    <span class="result-league"
                                                        >{u.league}</span
                                                    >
                                                {/if}
                                            </button>
                                        </li>
                                    {/each}
                                </ul>
                            {/if}
                        {:else if selCategory}
                            <div class="subcat-list">
                                {#each selCategory.subcategories as sub}
                                    <button
                                        class="subcat-btn"
                                        class:active={selSubLabel === sub.label}
                                        onclick={() =>
                                            selectSubcategory(sub.label)}
                                        >{sub.label}</button
                                    >
                                {/each}
                            </div>
                        {/if}
                        {#if selSubcategory && selCatName !== "Unique"}
                            <div class="base-sep"></div>
                            <ul class="base-list">
                                {#each selSubcategory.bases as base}
                                    <li
                                        class="base-item"
                                        class:active={selBase === base.name}
                                    >
                                        <button
                                            class="base-btn"
                                            onclick={() =>
                                                selectBase(base.name)}
                                        >
                                            <span class="base-name"
                                                >{base.name}</span
                                            >
                                            <span class="base-lvl"
                                                >i{base.level_req}</span
                                            >
                                        </button>
                                    </li>
                                {/each}
                            </ul>
                        {/if}
                        {#if !categoryLoaded}
                            <p class="hint">Loading categories…</p>
                        {/if}
                    </div>

                    <!-- Right: config + mod picker / unique roll picker -->
                    <div class="craft-config">
                        {#if selCatName === "Unique"}
                            <!-- ── Unique roll picker ─────────────────────── -->
                            {#if !selUnique}
                                <p class="hint centered">
                                    Select a unique from the list to configure
                                    its rolls.
                                </p>
                            {:else}
                                <div class="unique-config">
                                    <div class="unique-header">
                                        <span class="unique-title"
                                            >{selUnique.name}</span
                                        >
                                        <span class="unique-base-lbl"
                                            >{selUnique.base_name}</span
                                        >
                                    </div>

                                    {#if selUnique.implicit_lines.some((l) => l.ranges.length > 0) || selUnique.explicit_lines.some((l) => l.ranges.length > 0)}
                                        <p
                                            class="hint"
                                            style="margin:4px 0 8px;font-size:0.72rem"
                                        >
                                            Adjust sliders to set exact roll
                                            values, then add to inventory.
                                        </p>
                                    {:else}
                                        <p
                                            class="hint"
                                            style="margin:4px 0 8px;font-size:0.72rem"
                                        >
                                            This unique has no variable rolls.
                                            Add it to inventory as-is.
                                        </p>
                                    {/if}

                                    {#if selUnique.implicit_lines.length > 0}
                                        <div class="mod-section">
                                            <h4>Implicits</h4>
                                            {#each selUnique.implicit_lines as line, li}
                                                <div
                                                    class="uniq-line"
                                                    class:line-unmapped={!line.is_mapped &&
                                                        !line.is_header}
                                                >
                                                    <div
                                                        class="uniq-line-text"
                                                        class:line-header={line.is_header}
                                                    >
                                                        {applyRollsToText(
                                                            line.text,
                                                            li,
                                                        )}
                                                    </div>
                                                    {#each line.ranges as range, ri}
                                                        {@const flatIdx =
                                                            uniqueLineOffsets[
                                                                li
                                                            ] + ri}
                                                        <div
                                                            class="uniq-range-row"
                                                        >
                                                            <input
                                                                type="range"
                                                                class="val-slider"
                                                                min={range[0]}
                                                                max={range[1]}
                                                                step={range[0] ===
                                                                    Math.trunc(
                                                                        range[0],
                                                                    ) &&
                                                                range[1] ===
                                                                    Math.trunc(
                                                                        range[1],
                                                                    )
                                                                    ? 1
                                                                    : 0.1}
                                                                value={uniqueRolls[
                                                                    flatIdx
                                                                ] ??
                                                                    (range[0] +
                                                                        range[1]) /
                                                                        2}
                                                                oninput={(
                                                                    e,
                                                                ) => {
                                                                    const v =
                                                                        parseFloat(
                                                                            (
                                                                                e.target as HTMLInputElement
                                                                            )
                                                                                .value,
                                                                        );
                                                                    if (
                                                                        !isNaN(
                                                                            v,
                                                                        )
                                                                    ) {
                                                                        const r =
                                                                            [
                                                                                ...uniqueRolls,
                                                                            ];
                                                                        r[
                                                                            flatIdx
                                                                        ] = v;
                                                                        uniqueRolls =
                                                                            r;
                                                                    }
                                                                }}
                                                            />
                                                            <input
                                                                type="number"
                                                                class="val-num"
                                                                min={range[0]}
                                                                max={range[1]}
                                                                value={uniqueRolls[
                                                                    flatIdx
                                                                ] ??
                                                                    (range[0] +
                                                                        range[1]) /
                                                                        2}
                                                                oninput={(
                                                                    e,
                                                                ) => {
                                                                    const v =
                                                                        parseFloat(
                                                                            (
                                                                                e.target as HTMLInputElement
                                                                            )
                                                                                .value,
                                                                        );
                                                                    if (
                                                                        !isNaN(
                                                                            v,
                                                                        )
                                                                    ) {
                                                                        const r =
                                                                            [
                                                                                ...uniqueRolls,
                                                                            ];
                                                                        r[
                                                                            flatIdx
                                                                        ] = v;
                                                                        uniqueRolls =
                                                                            r;
                                                                    }
                                                                }}
                                                            />
                                                            <span
                                                                class="val-range"
                                                                >({range[0]}-{range[1]})</span
                                                            >
                                                        </div>
                                                    {/each}
                                                </div>
                                            {/each}
                                        </div>
                                    {/if}

                                    {#if selUnique.explicit_lines.length > 0}
                                        {@const explOff =
                                            selUnique.implicit_lines.length}
                                        <div class="mod-section">
                                            <h4>Mods</h4>
                                            {#each selUnique.explicit_lines as line, eli}
                                                {@const li = explOff + eli}
                                                <div
                                                    class="uniq-line"
                                                    class:line-unmapped={!line.is_mapped &&
                                                        !line.is_header}
                                                >
                                                    <div
                                                        class="uniq-line-text"
                                                        class:line-header={line.is_header}
                                                    >
                                                        {applyRollsToText(
                                                            line.text,
                                                            li,
                                                        )}
                                                    </div>
                                                    {#each line.ranges as range, ri}
                                                        {@const flatIdx =
                                                            uniqueLineOffsets[
                                                                li
                                                            ] + ri}
                                                        <div
                                                            class="uniq-range-row"
                                                        >
                                                            <input
                                                                type="range"
                                                                class="val-slider"
                                                                min={range[0]}
                                                                max={range[1]}
                                                                step={range[0] ===
                                                                    Math.trunc(
                                                                        range[0],
                                                                    ) &&
                                                                range[1] ===
                                                                    Math.trunc(
                                                                        range[1],
                                                                    )
                                                                    ? 1
                                                                    : 0.1}
                                                                value={uniqueRolls[
                                                                    flatIdx
                                                                ] ??
                                                                    (range[0] +
                                                                        range[1]) /
                                                                        2}
                                                                oninput={(
                                                                    e,
                                                                ) => {
                                                                    const v =
                                                                        parseFloat(
                                                                            (
                                                                                e.target as HTMLInputElement
                                                                            )
                                                                                .value,
                                                                        );
                                                                    if (
                                                                        !isNaN(
                                                                            v,
                                                                        )
                                                                    ) {
                                                                        const r =
                                                                            [
                                                                                ...uniqueRolls,
                                                                            ];
                                                                        r[
                                                                            flatIdx
                                                                        ] = v;
                                                                        uniqueRolls =
                                                                            r;
                                                                    }
                                                                }}
                                                            />
                                                            <input
                                                                type="number"
                                                                class="val-num"
                                                                min={range[0]}
                                                                max={range[1]}
                                                                value={uniqueRolls[
                                                                    flatIdx
                                                                ] ??
                                                                    (range[0] +
                                                                        range[1]) /
                                                                        2}
                                                                oninput={(
                                                                    e,
                                                                ) => {
                                                                    const v =
                                                                        parseFloat(
                                                                            (
                                                                                e.target as HTMLInputElement
                                                                            )
                                                                                .value,
                                                                        );
                                                                    if (
                                                                        !isNaN(
                                                                            v,
                                                                        )
                                                                    ) {
                                                                        const r =
                                                                            [
                                                                                ...uniqueRolls,
                                                                            ];
                                                                        r[
                                                                            flatIdx
                                                                        ] = v;
                                                                        uniqueRolls =
                                                                            r;
                                                                    }
                                                                }}
                                                            />
                                                            <span
                                                                class="val-range"
                                                                >({range[0]}-{range[1]})</span
                                                            >
                                                        </div>
                                                    {/each}
                                                </div>
                                            {/each}
                                        </div>
                                    {/if}

                                    <div class="action-row">
                                        <button
                                            class="btn-add"
                                            disabled={addingUnique}
                                            onclick={addUniqueToBuild}
                                        >
                                            {addingUnique
                                                ? "Adding…"
                                                : "Add to Build Inventory"}
                                        </button>
                                        {#if addUniqueStatus}
                                            <span
                                                class="status"
                                                class:err={addUniqueStatus.startsWith(
                                                    "Error",
                                                )}
                                            >
                                                {addUniqueStatus}
                                            </span>
                                        {/if}
                                    </div>
                                </div>
                            {/if}
                        {:else if !selBase}
                            <p class="hint centered">
                                Select a category, subcategory, and base item to
                                begin crafting.
                            </p>
                        {:else if loadingBase}
                            <p class="hint">Loading mod data…</p>
                        {:else}
                            <!-- Config row -->
                            <div class="config-row">
                                <label>
                                    Name
                                    <input
                                        class="cfg-input wide"
                                        type="text"
                                        bind:value={itemName}
                                        placeholder="Leave blank for Normal"
                                    />
                                </label>
                                <label>
                                    iLvl
                                    <input
                                        class="cfg-input narrow"
                                        type="number"
                                        min="1"
                                        max="100"
                                        bind:value={itemLevel}
                                    />
                                </label>
                                <label>
                                    Quality
                                    <input
                                        class="cfg-input narrow"
                                        type="number"
                                        min="0"
                                        max="30"
                                        bind:value={quality}
                                    />%
                                </label>
                                <label>
                                    Influence
                                    <select
                                        class="cfg-input"
                                        bind:value={influence}
                                        onchange={onInfluenceChange}
                                    >
                                        <option value="">None</option>
                                        <option value="shaper">Shaper</option>
                                        <option value="elder">Elder</option>
                                        <option value="crusader"
                                            >Crusader</option
                                        >
                                        <option value="hunter">Hunter</option>
                                        <option value="redeemer"
                                            >Redeemer</option
                                        >
                                        <option value="warlord">Warlord</option>
                                    </select>
                                </label>
                            </div>

                            <!-- Base value overrides -->
                            {#if baseProps}
                                <div class="overrides">
                                    {#if baseProps.phys_damage_min != null}
                                        <div class="override-row">
                                            <span class="ov-label"
                                                >Phys Dmg</span
                                            >
                                            <input
                                                type="number"
                                                class="ov-input"
                                                min={baseProps.phys_damage_min}
                                                max={baseProps.phys_damage_min}
                                                bind:value={overridePhysMin}
                                            />
                                            –
                                            <input
                                                type="number"
                                                class="ov-input"
                                                min={baseProps.phys_damage_max}
                                                max={baseProps.phys_damage_max}
                                                bind:value={overridePhysMax}
                                            />
                                            <span class="ov-hint"
                                                >(base: {baseProps.phys_damage_min}–{baseProps.phys_damage_max})</span
                                            >
                                        </div>
                                    {/if}
                                    {#if baseProps.armour_max != null}
                                        <div class="override-row">
                                            <span class="ov-label">Armour</span>
                                            <input
                                                type="number"
                                                class="ov-input"
                                                min={baseProps.armour_min ?? 0}
                                                max={baseProps.armour_max}
                                                bind:value={overrideArmour}
                                            />
                                            <span class="ov-hint"
                                                >(max {baseProps.armour_max})</span
                                            >
                                        </div>
                                    {/if}
                                    {#if baseProps.evasion_max != null}
                                        <div class="override-row">
                                            <span class="ov-label">Evasion</span
                                            >
                                            <input
                                                type="number"
                                                class="ov-input"
                                                min={baseProps.evasion_min ?? 0}
                                                max={baseProps.evasion_max}
                                                bind:value={overrideEvasion}
                                            />
                                            <span class="ov-hint"
                                                >(max {baseProps.evasion_max})</span
                                            >
                                        </div>
                                    {/if}
                                    {#if baseProps.energy_shield_max != null}
                                        <div class="override-row">
                                            <span class="ov-label"
                                                >Energy Shield</span
                                            >
                                            <input
                                                type="number"
                                                class="ov-input"
                                                min={baseProps.energy_shield_min ??
                                                    0}
                                                max={baseProps.energy_shield_max}
                                                bind:value={overrideES}
                                            />
                                            <span class="ov-hint"
                                                >(max {baseProps.energy_shield_max})</span
                                            >
                                        </div>
                                    {/if}
                                </div>
                            {/if}

                            {#if baseModGroups}
                                <!-- Implicits -->
                                {#if baseModGroups.implicits.length > 0}
                                    <div class="mod-section">
                                        <h4>
                                            Implicits ({countSel(
                                                implicitSel,
                                            )}/{baseModGroups.implicits.length})
                                        </h4>
                                        {#each baseModGroups.implicits as group, gi}
                                            {@const sel = implicitSel[gi]}
                                            <div
                                                class="mod-row"
                                                class:active={sel != null}
                                            >
                                                <span
                                                    class="mod-dn"
                                                    title={group.display_name}
                                                    >{group.display_name}</span
                                                >
                                                <select
                                                    class="tier-sel"
                                                    value={sel?.tierIdx ?? -1}
                                                    onchange={(e) => {
                                                        const v = parseInt(
                                                            (
                                                                e.target as HTMLSelectElement
                                                            ).value,
                                                        );
                                                        implicitSel =
                                                            setTierFor(
                                                                implicitSel,
                                                                baseModGroups!
                                                                    .implicits,
                                                                gi,
                                                                v,
                                                            );
                                                    }}
                                                >
                                                    <option value={-1}
                                                        >— None —</option
                                                    >
                                                    {#each group.tiers as tier, ti}
                                                        <option value={ti}
                                                            >{tierLabel(
                                                                tier,
                                                            )}</option
                                                        >
                                                    {/each}
                                                </select>
                                            </div>
                                            {#if sel != null}
                                                {@const activeTier =
                                                    group.tiers[sel.tierIdx]}
                                                {#if activeTier}
                                                    <div class="mod-values">
                                                        {#each activeTier.stats as stat, si}
                                                            <div
                                                                class="val-row"
                                                            >
                                                                <span
                                                                    class="val-label"
                                                                    >{stat.stat_id.replace(
                                                                        /_/g,
                                                                        "\u00a0",
                                                                    )}</span
                                                                >
                                                                {#if stat.min < stat.max}
                                                                    <input
                                                                        type="range"
                                                                        class="val-slider"
                                                                        min={stat.min}
                                                                        max={stat.max}
                                                                        step={1}
                                                                        value={sel
                                                                            .values[
                                                                            si
                                                                        ] ??
                                                                            midpoint(
                                                                                stat.min,
                                                                                stat.max,
                                                                            )}
                                                                        oninput={(
                                                                            e,
                                                                        ) => {
                                                                            implicitSel =
                                                                                setValueFor(
                                                                                    implicitSel,
                                                                                    gi,
                                                                                    si,
                                                                                    parseFloat(
                                                                                        (
                                                                                            e.target as HTMLInputElement
                                                                                        )
                                                                                            .value,
                                                                                    ),
                                                                                );
                                                                        }}
                                                                    />
                                                                {/if}
                                                                <input
                                                                    type="number"
                                                                    class="val-num"
                                                                    min={stat.min}
                                                                    max={stat.max}
                                                                    value={sel
                                                                        .values[
                                                                        si
                                                                    ] ??
                                                                        midpoint(
                                                                            stat.min,
                                                                            stat.max,
                                                                        )}
                                                                    oninput={(
                                                                        e,
                                                                    ) => {
                                                                        const v =
                                                                            parseFloat(
                                                                                (
                                                                                    e.target as HTMLInputElement
                                                                                )
                                                                                    .value,
                                                                            );
                                                                        if (
                                                                            !isNaN(
                                                                                v,
                                                                            )
                                                                        )
                                                                            implicitSel =
                                                                                setValueFor(
                                                                                    implicitSel,
                                                                                    gi,
                                                                                    si,
                                                                                    v,
                                                                                );
                                                                    }}
                                                                />
                                                                <span
                                                                    class="val-range"
                                                                    >({stat.min}–{stat.max})</span
                                                                >
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
                                    <input
                                        class="filter-input"
                                        type="text"
                                        placeholder="Filter prefixes…"
                                        bind:value={filterPrefix}
                                    />
                                    <div class="mod-list">
                                        {#each filteredIndices(baseModGroups.prefixes, filterPrefix) as gi}
                                            {@const group =
                                                baseModGroups.prefixes[gi]}
                                            {@const sel = prefixSel[gi]}
                                            <div
                                                class="mod-row"
                                                class:active={sel != null}
                                            >
                                                <span
                                                    class="mod-dn"
                                                    title={group.display_name}
                                                    >{group.display_name}</span
                                                >
                                                <select
                                                    class="tier-sel"
                                                    value={sel?.tierIdx ?? -1}
                                                    disabled={sel == null &&
                                                        countSel(prefixSel) >=
                                                            3}
                                                    onchange={(e) => {
                                                        const v = parseInt(
                                                            (
                                                                e.target as HTMLSelectElement
                                                            ).value,
                                                        );
                                                        prefixSel = setTierFor(
                                                            prefixSel,
                                                            baseModGroups!
                                                                .prefixes,
                                                            gi,
                                                            v,
                                                        );
                                                    }}
                                                >
                                                    <option value={-1}
                                                        >— None —</option
                                                    >
                                                    {#each group.tiers as tier, ti}
                                                        <option value={ti}
                                                            >{tierLabel(
                                                                tier,
                                                            )}</option
                                                        >
                                                    {/each}
                                                </select>
                                            </div>
                                            {#if sel != null}
                                                {@const activeTier =
                                                    group.tiers[sel.tierIdx]}
                                                {#if activeTier}
                                                    <div class="mod-values">
                                                        {#each activeTier.stats as stat, si}
                                                            <div
                                                                class="val-row"
                                                            >
                                                                <span
                                                                    class="val-label"
                                                                    >{stat.stat_id.replace(
                                                                        /_/g,
                                                                        "\u00a0",
                                                                    )}</span
                                                                >
                                                                {#if stat.min < stat.max}
                                                                    <input
                                                                        type="range"
                                                                        class="val-slider"
                                                                        min={stat.min}
                                                                        max={stat.max}
                                                                        step={1}
                                                                        value={sel
                                                                            .values[
                                                                            si
                                                                        ] ??
                                                                            midpoint(
                                                                                stat.min,
                                                                                stat.max,
                                                                            )}
                                                                        oninput={(
                                                                            e,
                                                                        ) => {
                                                                            prefixSel =
                                                                                setValueFor(
                                                                                    prefixSel,
                                                                                    gi,
                                                                                    si,
                                                                                    parseFloat(
                                                                                        (
                                                                                            e.target as HTMLInputElement
                                                                                        )
                                                                                            .value,
                                                                                    ),
                                                                                );
                                                                        }}
                                                                    />
                                                                {/if}
                                                                <input
                                                                    type="number"
                                                                    class="val-num"
                                                                    min={stat.min}
                                                                    max={stat.max}
                                                                    value={sel
                                                                        .values[
                                                                        si
                                                                    ] ??
                                                                        midpoint(
                                                                            stat.min,
                                                                            stat.max,
                                                                        )}
                                                                    oninput={(
                                                                        e,
                                                                    ) => {
                                                                        const v =
                                                                            parseFloat(
                                                                                (
                                                                                    e.target as HTMLInputElement
                                                                                )
                                                                                    .value,
                                                                            );
                                                                        if (
                                                                            !isNaN(
                                                                                v,
                                                                            )
                                                                        )
                                                                            prefixSel =
                                                                                setValueFor(
                                                                                    prefixSel,
                                                                                    gi,
                                                                                    si,
                                                                                    v,
                                                                                );
                                                                    }}
                                                                />
                                                                <span
                                                                    class="val-range"
                                                                    >({stat.min}–{stat.max})</span
                                                                >
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
                                    <input
                                        class="filter-input"
                                        type="text"
                                        placeholder="Filter suffixes…"
                                        bind:value={filterSuffix}
                                    />
                                    <div class="mod-list">
                                        {#each filteredIndices(baseModGroups.suffixes, filterSuffix) as gi}
                                            {@const group =
                                                baseModGroups.suffixes[gi]}
                                            {@const sel = suffixSel[gi]}
                                            <div
                                                class="mod-row"
                                                class:active={sel != null}
                                            >
                                                <span
                                                    class="mod-dn"
                                                    title={group.display_name}
                                                    >{group.display_name}</span
                                                >
                                                <select
                                                    class="tier-sel"
                                                    value={sel?.tierIdx ?? -1}
                                                    disabled={sel == null &&
                                                        countSel(suffixSel) >=
                                                            3}
                                                    onchange={(e) => {
                                                        const v = parseInt(
                                                            (
                                                                e.target as HTMLSelectElement
                                                            ).value,
                                                        );
                                                        suffixSel = setTierFor(
                                                            suffixSel,
                                                            baseModGroups!
                                                                .suffixes,
                                                            gi,
                                                            v,
                                                        );
                                                    }}
                                                >
                                                    <option value={-1}
                                                        >— None —</option
                                                    >
                                                    {#each group.tiers as tier, ti}
                                                        <option value={ti}
                                                            >{tierLabel(
                                                                tier,
                                                            )}</option
                                                        >
                                                    {/each}
                                                </select>
                                            </div>
                                            {#if sel != null}
                                                {@const activeTier =
                                                    group.tiers[sel.tierIdx]}
                                                {#if activeTier}
                                                    <div class="mod-values">
                                                        {#each activeTier.stats as stat, si}
                                                            <div
                                                                class="val-row"
                                                            >
                                                                <span
                                                                    class="val-label"
                                                                    >{stat.stat_id.replace(
                                                                        /_/g,
                                                                        "\u00a0",
                                                                    )}</span
                                                                >
                                                                {#if stat.min < stat.max}
                                                                    <input
                                                                        type="range"
                                                                        class="val-slider"
                                                                        min={stat.min}
                                                                        max={stat.max}
                                                                        step={1}
                                                                        value={sel
                                                                            .values[
                                                                            si
                                                                        ] ??
                                                                            midpoint(
                                                                                stat.min,
                                                                                stat.max,
                                                                            )}
                                                                        oninput={(
                                                                            e,
                                                                        ) => {
                                                                            suffixSel =
                                                                                setValueFor(
                                                                                    suffixSel,
                                                                                    gi,
                                                                                    si,
                                                                                    parseFloat(
                                                                                        (
                                                                                            e.target as HTMLInputElement
                                                                                        )
                                                                                            .value,
                                                                                    ),
                                                                                );
                                                                        }}
                                                                    />
                                                                {/if}
                                                                <input
                                                                    type="number"
                                                                    class="val-num"
                                                                    min={stat.min}
                                                                    max={stat.max}
                                                                    value={sel
                                                                        .values[
                                                                        si
                                                                    ] ??
                                                                        midpoint(
                                                                            stat.min,
                                                                            stat.max,
                                                                        )}
                                                                    oninput={(
                                                                        e,
                                                                    ) => {
                                                                        const v =
                                                                            parseFloat(
                                                                                (
                                                                                    e.target as HTMLInputElement
                                                                                )
                                                                                    .value,
                                                                            );
                                                                        if (
                                                                            !isNaN(
                                                                                v,
                                                                            )
                                                                        )
                                                                            suffixSel =
                                                                                setValueFor(
                                                                                    suffixSel,
                                                                                    gi,
                                                                                    si,
                                                                                    v,
                                                                                );
                                                                    }}
                                                                />
                                                                <span
                                                                    class="val-range"
                                                                    >({stat.min}–{stat.max})</span
                                                                >
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
                                    <input
                                        class="filter-input"
                                        type="text"
                                        placeholder="Filter crafted mods…"
                                        bind:value={filterCrafted}
                                    />
                                    <div class="mod-list">
                                        {#each filteredIndices(baseModGroups.crafted, filterCrafted) as gi}
                                            {@const group =
                                                baseModGroups.crafted[gi]}
                                            {@const sel = craftedSel[gi]}
                                            <div
                                                class="mod-row"
                                                class:active={sel != null}
                                            >
                                                <span
                                                    class="mod-dn"
                                                    title={group.display_name}
                                                    >{group.display_name}</span
                                                >
                                                <select
                                                    class="tier-sel"
                                                    value={sel?.tierIdx ?? -1}
                                                    disabled={sel == null &&
                                                        countSel(craftedSel) >=
                                                            1}
                                                    onchange={(e) => {
                                                        const v = parseInt(
                                                            (
                                                                e.target as HTMLSelectElement
                                                            ).value,
                                                        );
                                                        craftedSel = setTierFor(
                                                            craftedSel,
                                                            baseModGroups!
                                                                .crafted,
                                                            gi,
                                                            v,
                                                        );
                                                    }}
                                                >
                                                    <option value={-1}
                                                        >— None —</option
                                                    >
                                                    {#each group.tiers as tier, ti}
                                                        <option value={ti}
                                                            >{tierLabel(
                                                                tier,
                                                            )}</option
                                                        >
                                                    {/each}
                                                </select>
                                            </div>
                                            {#if sel != null}
                                                {@const activeTier =
                                                    group.tiers[sel.tierIdx]}
                                                {#if activeTier}
                                                    <div class="mod-values">
                                                        {#each activeTier.stats as stat, si}
                                                            <div
                                                                class="val-row"
                                                            >
                                                                <span
                                                                    class="val-label"
                                                                    >{stat.stat_id.replace(
                                                                        /_/g,
                                                                        "\u00a0",
                                                                    )}</span
                                                                >
                                                                {#if stat.min < stat.max}
                                                                    <input
                                                                        type="range"
                                                                        class="val-slider"
                                                                        min={stat.min}
                                                                        max={stat.max}
                                                                        step={1}
                                                                        value={sel
                                                                            .values[
                                                                            si
                                                                        ] ??
                                                                            midpoint(
                                                                                stat.min,
                                                                                stat.max,
                                                                            )}
                                                                        oninput={(
                                                                            e,
                                                                        ) => {
                                                                            craftedSel =
                                                                                setValueFor(
                                                                                    craftedSel,
                                                                                    gi,
                                                                                    si,
                                                                                    parseFloat(
                                                                                        (
                                                                                            e.target as HTMLInputElement
                                                                                        )
                                                                                            .value,
                                                                                    ),
                                                                                );
                                                                        }}
                                                                    />
                                                                {/if}
                                                                <input
                                                                    type="number"
                                                                    class="val-num"
                                                                    min={stat.min}
                                                                    max={stat.max}
                                                                    value={sel
                                                                        .values[
                                                                        si
                                                                    ] ??
                                                                        midpoint(
                                                                            stat.min,
                                                                            stat.max,
                                                                        )}
                                                                    oninput={(
                                                                        e,
                                                                    ) => {
                                                                        const v =
                                                                            parseFloat(
                                                                                (
                                                                                    e.target as HTMLInputElement
                                                                                )
                                                                                    .value,
                                                                            );
                                                                        if (
                                                                            !isNaN(
                                                                                v,
                                                                            )
                                                                        )
                                                                            craftedSel =
                                                                                setValueFor(
                                                                                    craftedSel,
                                                                                    gi,
                                                                                    si,
                                                                                    v,
                                                                                );
                                                                    }}
                                                                />
                                                                <span
                                                                    class="val-range"
                                                                    >({stat.min}–{stat.max})</span
                                                                >
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
                                <button
                                    class="btn-add"
                                    disabled={!selBase || adding}
                                    onclick={addToBuild}
                                >
                                    {adding
                                        ? "Adding…"
                                        : "Add to Build Inventory"}
                                </button>
                                {#if addStatus}
                                    <span
                                        class="status"
                                        class:err={addStatus.startsWith(
                                            "Error",
                                        )}
                                    >
                                        {addStatus}
                                    </span>
                                {/if}
                            </div>
                        {/if}
                    </div>
                </div>
            </div>

            <!-- Always-visible preview panel -->
            <div class="preview-panel">
                <h3>Preview</h3>
                {#if selCatName === "Unique" && selUnique && !hoveredInvDetail}
                    <!-- PoE-style unique item tooltip preview -->
                    <div class="poe-item-box">
                        <div class="poe-item-name">{selUnique.name}</div>
                        <div class="poe-item-base">{selUnique.base_name}</div>
                        {#if selUnique.influences !== 0}
                            <div class="poe-inf-row">
                                {#each influenceList(selUnique.influences) as inf}
                                    <span class="poe-inf-badge">{inf}</span>
                                {/each}
                            </div>
                        {/if}
                        {#if selUnique.league}
                            <div class="poe-league-lbl">{selUnique.league}</div>
                        {/if}
                        {#if selUnique.base_props}
                            {@const bp = selUnique.base_props}
                            <hr class="pdiv" />
                            {#if bp.level_req > 0}
                                <div class="poe-req">
                                    Requires Level <strong
                                        >{bp.level_req}</strong
                                    >
                                </div>
                            {/if}
                            {#if bp.armour_max != null}
                                <div class="poe-stat">
                                    Armour: <span class="poe-val"
                                        >{bp.armour_max}</span
                                    >
                                </div>
                            {/if}
                            {#if bp.evasion_max != null}
                                <div class="poe-stat">
                                    Evasion: <span class="poe-val"
                                        >{bp.evasion_max}</span
                                    >
                                </div>
                            {/if}
                            {#if bp.energy_shield_max != null}
                                <div class="poe-stat">
                                    Energy Shield: <span class="poe-val"
                                        >{bp.energy_shield_max}</span
                                    >
                                </div>
                            {/if}
                            {#if bp.phys_damage_min != null}
                                <div class="poe-stat">
                                    Physical Damage: <span class="poe-val"
                                        >{bp.phys_damage_min}–{bp.phys_damage_max}</span
                                    >
                                    {#if bp.attack_time_ms}<span
                                            class="poe-aps"
                                        >
                                            @ {(
                                                1000 / bp.attack_time_ms
                                            ).toFixed(2)} APS</span
                                        >{/if}
                                </div>
                            {/if}
                        {/if}
                        {#if selUnique.implicit_lines.length > 0}
                            <hr class="pdiv" />
                            {#each selUnique.implicit_lines as line, li}
                                {#if line.is_header}
                                    <div class="poe-line-header">
                                        {line.text}
                                    </div>
                                {:else}
                                    <div
                                        class="poe-mod"
                                        class:poe-mod-unmapped={!line.is_mapped}
                                    >
                                        {applyRollsToText(line.text, li)}
                                        {#if !line.is_mapped}<span
                                                class="poe-unmapped-tag"
                                                title="Not yet in stat engine"
                                                >?</span
                                            >{/if}
                                    </div>
                                {/if}
                            {/each}
                        {/if}
                        {#if selUnique.explicit_lines.length > 0}
                            <hr class="pdiv" />
                            {@const explOff = selUnique.implicit_lines.length}
                            {#each selUnique.explicit_lines as line, eli}
                                {@const li = explOff + eli}
                                {#if line.is_header}
                                    <div class="poe-line-header">
                                        {line.text}
                                    </div>
                                {:else}
                                    <div
                                        class="poe-mod"
                                        class:poe-mod-unmapped={!line.is_mapped}
                                    >
                                        {applyRollsToText(line.text, li)}
                                        {#if !line.is_mapped}<span
                                                class="poe-unmapped-tag"
                                                title="Not yet in stat engine"
                                                >?</span
                                            >{/if}
                                    </div>
                                {/if}
                            {/each}
                        {/if}
                    </div>
                {:else if selBase && baseModGroups && !hoveredInvDetail}
                    {@const totalExplicit =
                        countSel(prefixSel) +
                        countSel(suffixSel) +
                        countSel(craftedSel)}
                    <div class="preview-item">
                        <span
                            class="preview-name"
                            class:name-rare={totalExplicit > 2}
                            class:name-magic={totalExplicit > 0 &&
                                totalExplicit <= 2}
                            >{itemName.trim() || selBase}</span
                        >
                        <div class="preview-base">
                            {selBase}
                            {#if influence}<span class="preview-inf">
                                    · {influence.charAt(0).toUpperCase() +
                                        influence.slice(1)}</span
                                >{/if}
                        </div>
                        {#if baseProps}
                            {#if baseProps.armour_max != null}
                                <div class="preview-stat">
                                    Armour: <strong
                                        >{(
                                            overrideArmour ??
                                            baseProps.armour_max
                                        )?.toFixed(0)}</strong
                                    >
                                </div>
                            {/if}
                            {#if baseProps.evasion_max != null}
                                <div class="preview-stat">
                                    Evasion: <strong
                                        >{(
                                            overrideEvasion ??
                                            baseProps.evasion_max
                                        )?.toFixed(0)}</strong
                                    >
                                </div>
                            {/if}
                            {#if baseProps.energy_shield_max != null}
                                <div class="preview-stat">
                                    Energy Shield: <strong
                                        >{(
                                            overrideES ??
                                            baseProps.energy_shield_max
                                        )?.toFixed(0)}</strong
                                    >
                                </div>
                            {/if}
                            {#if baseProps.phys_damage_min != null}
                                <div class="preview-stat">
                                    Phys: <strong
                                        >{overridePhysMin ??
                                            baseProps.phys_damage_min}–{overridePhysMax ??
                                            baseProps.phys_damage_max}</strong
                                    >
                                    {#if baseProps.attack_time_ms}<span
                                            class="gray"
                                        >
                                            @ {(
                                                1000 / baseProps.attack_time_ms
                                            ).toFixed(2)} APS</span
                                        >{/if}
                                </div>
                            {/if}
                        {/if}
                        {#if countSel(implicitSel) > 0}
                            <hr class="pdiv" />
                            {#each Object.entries(implicitSel) as [idxStr, s]}
                                {@const group =
                                    baseModGroups.implicits[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod implicit">
                                        {fillTemplate(
                                            group.display_name,
                                            s.values,
                                        )}
                                    </div>
                                {/if}
                            {/each}
                        {/if}
                        {#if totalExplicit > 0}
                            <hr class="pdiv" />
                            {#each Object.entries(prefixSel) as [idxStr, s]}
                                {@const group =
                                    baseModGroups.prefixes[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod">
                                        {fillTemplate(
                                            group.display_name,
                                            s.values,
                                        )}
                                    </div>
                                {/if}
                            {/each}
                            {#each Object.entries(suffixSel) as [idxStr, s]}
                                {@const group =
                                    baseModGroups.suffixes[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod">
                                        {fillTemplate(
                                            group.display_name,
                                            s.values,
                                        )}
                                    </div>
                                {/if}
                            {/each}
                            {#each Object.entries(craftedSel) as [idxStr, s]}
                                {@const group =
                                    baseModGroups.crafted[parseInt(idxStr)]}
                                {#if group}
                                    <div class="preview-mod crafted">
                                        {fillTemplate(
                                            group.display_name,
                                            s.values,
                                        )}
                                    </div>
                                {/if}
                            {/each}
                        {/if}
                    </div>
                {:else if activePreviewDetail}
                    {@const d = activePreviewDetail}
                    <div
                        class="poe-item-box"
                        class:poe-item-magic={d.rarity === "Magic"}
                        class:poe-item-rare={d.rarity === "Rare"}
                    >
                        <div
                            class="poe-item-name"
                            class:poe-name-magic={d.rarity === "Magic"}
                            class:poe-name-rare={d.rarity === "Rare"}
                        >
                            {d.name}
                        </div>
                        <div class="poe-item-base">{d.base_name}</div>
                        {#if d.influences !== 0}
                            <div class="poe-inf-row">
                                {#each influenceList(d.influences) as inf}
                                    <span class="poe-inf-badge">{inf}</span>
                                {/each}
                            </div>
                        {/if}
                        {#if d.corrupted}
                            <div
                                style="color:#d02020;font-size:0.75rem;margin-top:2px"
                            >
                                Corrupted
                            </div>
                        {/if}
                        {#if d.mirrored}
                            <div
                                style="color:#8888cc;font-size:0.75rem;margin-top:2px"
                            >
                                Mirrored
                            </div>
                        {/if}
                        {#if d.req_level > 0 || d.req_str > 0 || d.req_dex > 0 || d.req_int > 0}
                            <hr class="pdiv" />
                            <div class="poe-req">
                                {#if d.req_level > 0}Requires Level <strong
                                        >{d.req_level}</strong
                                    >{/if}
                                {#if d.req_str > 0}&nbsp;<span
                                        style="color:#c77a3a"
                                        >{d.req_str} Str</span
                                    >{/if}
                                {#if d.req_dex > 0}&nbsp;<span
                                        style="color:#8aba44"
                                        >{d.req_dex} Dex</span
                                    >{/if}
                                {#if d.req_int > 0}&nbsp;<span
                                        style="color:#7ba2d8"
                                        >{d.req_int} Int</span
                                    >{/if}
                            </div>
                        {/if}
                        {#if d.phys_damage_min != null}
                            <hr class="pdiv" />
                            <div class="poe-stat">
                                Physical Damage: <span class="poe-val"
                                    >{d.phys_damage_min!.toFixed(
                                        0,
                                    )}–{d.phys_damage_max!.toFixed(0)}</span
                                >
                                {#if d.attacks_per_second != null}<span
                                        class="poe-aps"
                                    >
                                        @ {d.attacks_per_second.toFixed(2)} APS</span
                                    >{/if}
                            </div>
                            {#if d.crit_chance != null}
                                <div class="poe-stat">
                                    Critical Strike Chance: <span
                                        class="poe-val"
                                        >{d.crit_chance.toFixed(2)}%</span
                                    >
                                </div>
                            {/if}
                            {#if d.phys_dps != null && d.phys_dps > 0}
                                <div class="poe-stat">
                                    Physical DPS: <span class="poe-val"
                                        >{d.phys_dps.toFixed(1)}</span
                                    >
                                </div>
                            {/if}
                            {#if d.ele_dps != null && d.ele_dps > 0}
                                <div class="poe-stat">
                                    Elemental DPS: <span class="poe-val"
                                        >{d.ele_dps.toFixed(1)}</span
                                    >
                                </div>
                            {/if}
                            {#if d.total_dps != null}
                                <div class="poe-stat">
                                    Total DPS: <span class="poe-val"
                                        >{d.total_dps.toFixed(1)}</span
                                    >
                                </div>
                            {/if}
                        {:else if d.armour != null || d.evasion != null || d.energy_shield != null}
                            <hr class="pdiv" />
                            {#if d.armour != null}
                                <div class="poe-stat">
                                    Armour: <span class="poe-val"
                                        >{d.armour.toFixed(0)}</span
                                    >
                                </div>
                            {/if}
                            {#if d.evasion != null}
                                <div class="poe-stat">
                                    Evasion Rating: <span class="poe-val"
                                        >{d.evasion.toFixed(0)}</span
                                    >
                                </div>
                            {/if}
                            {#if d.energy_shield != null}
                                <div class="poe-stat">
                                    Energy Shield: <span class="poe-val"
                                        >{d.energy_shield.toFixed(0)}</span
                                    >
                                </div>
                            {/if}
                            {#if d.block != null}
                                <div class="poe-stat">
                                    Chance to Block: <span class="poe-val"
                                        >{d.block}%</span
                                    >
                                </div>
                            {/if}
                        {/if}
                        {#if d.enchant_lines.length > 0}
                            <hr class="pdiv" />
                            {#each d.enchant_lines as line}
                                <div class="poe-mod poe-mod-enchant">
                                    {line.text}
                                </div>
                            {/each}
                        {/if}
                        {#if d.implicit_lines.length > 0}
                            <hr class="pdiv" />
                            {#each d.implicit_lines as line}
                                <div class="poe-mod">{line.text}</div>
                            {/each}
                        {/if}
                        {#if d.explicit_lines.length > 0}
                            <hr class="pdiv" />
                            {#each d.explicit_lines as line}
                                <div
                                    class="poe-mod"
                                    class:poe-mod-crafted={line.kind ===
                                        "crafted"}
                                    class:poe-mod-fractured={line.kind ===
                                        "fractured"}
                                >
                                    {line.text}
                                </div>
                            {/each}
                        {/if}
                        {#if d.item_level > 0}
                            <div
                                style="color:#555;font-size:0.72rem;margin-top:6px"
                            >
                                Item Level: {d.item_level}{d.quality > 0
                                    ? ` · Quality: ${d.quality}%`
                                    : ""}
                            </div>
                        {/if}
                    </div>
                {:else if previewEquippedItem && !hoveredInvDetail}
                    <p class="hint">Loading…</p>
                {:else}
                    <p class="hint">Hover an item to preview it.</p>
                {/if}
            </div>
        </section>
    </div>
</main>

{#if tooltipData}
    <div class="item-tooltip" style="left:{tooltipX}px;top:{tooltipY}px">
        {#if tooltipData.kind === "equipped"}
            <span class="tt-name">{tooltipData.name}</span>
            <span class="tt-base">{tooltipData.base_name}</span>
            <hr class="tt-divider" />
            {#if tooltipData.total_dps != null}
                <div class="tt-row">
                    <span class="tt-label">DPS</span><span class="tt-value"
                        >{tooltipData.total_dps.toFixed(1)}</span
                    >
                </div>
            {/if}
            {#if tooltipData.armour != null}
                <div class="tt-row">
                    <span class="tt-label">Armour</span><span class="tt-value"
                        >{tooltipData.armour.toFixed(0)}</span
                    >
                </div>
            {/if}
            {#if tooltipData.evasion != null}
                <div class="tt-row">
                    <span class="tt-label">Evasion</span><span class="tt-value"
                        >{tooltipData.evasion.toFixed(0)}</span
                    >
                </div>
            {/if}
            {#if tooltipData.energy_shield != null}
                <div class="tt-row">
                    <span class="tt-label">Energy Shield</span><span
                        class="tt-value"
                        >{tooltipData.energy_shield.toFixed(0)}</span
                    >
                </div>
            {/if}
            <div class="tt-row">
                <span class="tt-label">Mods</span><span class="tt-value"
                    >{tooltipData.mod_count}</span
                >
            </div>
            <div class="tt-class">
                {tooltipData.item_class} — {tooltipData.slot}
            </div>
        {/if}
    </div>
{/if}

<style>
    :global(body) {
        margin: 0;
        background-color: #0e0e10;
        color: #e0d6c2;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    }

    .items-page {
        display: flex;
        height: 100vh;
        overflow: hidden;
    }

    .page-body {
        flex: 1;
        min-width: 0;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .content {
        display: flex;
        gap: 24px;
        padding: 16px 24px;
        flex: 1;
        min-height: 0;
        overflow: hidden;
    }

    /* Equipment grid */
    .equip-panel {
        flex: 0 0 220px;
    }

    .equip-panel h3 {
        margin: 0 0 12px;
        font-size: 0.95rem;
        text-transform: uppercase;
        letter-spacing: 0.07em;
        color: #c8a96e;
    }

    .equip-grid {
        display: grid;
        grid-template-columns: repeat(3, 64px);
        grid-template-rows: repeat(6, 56px);
        gap: 4px;
    }

    .slot-btn {
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 4px;
        color: #e0d6c2;
        cursor: pointer;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        font-size: 0.65rem;
        padding: 2px;
        overflow: hidden;
        transition: border-color 0.15s;
    }

    .slot-btn:hover {
        border-color: #c8a96e;
    }
    .slot-btn.selected {
        border-color: #c8a96e;
        background: #22201a;
    }
    .slot-btn.occupied {
        border-color: #7a6a3e;
    }

    .slot-label {
        color: #555;
        font-size: 0.6rem;
        text-transform: uppercase;
    }
    .slot-name {
        color: #c8a96e;
        font-size: 0.6rem;
        text-align: center;
        line-height: 1.2;
    }
    .slot-stat {
        color: #9eb9d4;
        font-size: 0.6rem;
    }

    .slot-actions {
        margin-top: 8px;
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.8rem;
    }

    .equipped-label {
        color: #c8a96e;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .empty-label {
        color: #555;
    }

    /* Center panels */
    .inv-panel-outer {
        flex: 0 0 240px;
        display: flex;
        flex-direction: column;
        min-width: 0;
        min-height: 0;
        overflow: hidden;
    }

    .inv-panel-outer h3 {
        margin: 0 0 12px;
        font-size: 0.95rem;
        text-transform: uppercase;
        letter-spacing: 0.07em;
        color: #c8a96e;
    }

    .craft-panel-outer {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* Always-visible preview panel */
    .preview-panel {
        flex: 0 0 210px;
        background: #111115;
        border: 1px solid #2a2a30;
        border-radius: 6px;
        padding: 10px 12px;
        overflow-y: auto;
        align-self: flex-start;
        max-height: calc(100vh - 100px);
    }

    .preview-panel h3 {
        margin: 0 0 10px;
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.07em;
        color: #c8a96e;
    }

    .preview-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .preview-name {
        font-size: 0.88rem;
        font-weight: 600;
        color: #e0d6c2;
    }
    .preview-name.name-magic {
        color: #8888ff;
    }
    .preview-name.name-rare {
        color: #ffd700;
    }
    .preview-base {
        color: #888;
        font-size: 0.72rem;
        margin-bottom: 3px;
    }
    .preview-inf {
        color: #6a9aca;
    }
    .preview-stat {
        color: #c8b88a;
        font-size: 0.78rem;
    }
    .preview-stat .gray {
        color: #666;
    }
    .pdiv {
        border: none;
        border-top: 1px solid #2e2c24;
        margin: 5px 0;
    }
    .preview-mod {
        color: #d0c0a0;
        font-size: 0.76rem;
    }
    .preview-mod.implicit {
        color: #88aacc;
    }
    .preview-mod.crafted {
        color: #a8c8f8;
    }

    /* Inventory panel */
    .inv-panel {
        flex: 1;
        overflow-y: auto;
        min-height: 0;
        display: flex;
        flex-direction: column;
        gap: 0;
    }

    /* Group separator */
    .inv-group {
        display: flex;
        flex-direction: column;
    }

    .inv-group-header {
        font-size: 0.65rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: #555;
        padding: 6px 6px 3px;
        border-top: 1px solid #2a2a30;
        margin-top: 4px;
    }

    .inv-group:first-child .inv-group-header {
        margin-top: 0;
        border-top: none;
    }

    .inv-item {
        display: flex;
        align-items: center;
        gap: 8px;
        background: #1a1a1e;
        border: 1px solid #2a2a30;
        border-left: 3px solid #2a2a30;
        border-radius: 4px;
        padding: 6px 8px;
        margin: 1px 2px;
        cursor: default;
    }

    .inv-item:hover {
        border-color: #4a4a52;
        border-left-color: #6a6a7a;
        background: #1e1e24;
    }

    /* Equipped item — gold left-border + subtle gold tint */
    .inv-item-equipped {
        border-left-color: #c8a96e !important;
        background: #1c1a14;
    }

    .inv-item-equipped:hover {
        background: #221e16;
    }

    /* Unequipped item compatible with the selected slot — green tint */
    .inv-item-compatible {
        border-left-color: #5a8a40 !important;
    }

    .inv-item-info {
        flex: 1;
        min-width: 0;
    }

    .inv-name-row {
        display: flex;
        align-items: center;
        gap: 6px;
        min-width: 0;
    }

    .inv-name {
        font-size: 0.82rem;
        font-weight: 500;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        flex: 1;
        min-width: 0;
    }

    .inv-equipped-badge {
        font-size: 0.6rem;
        background: #3a2e14;
        border: 1px solid #c8a96e66;
        border-radius: 3px;
        color: #c8a96e;
        padding: 1px 4px;
        flex-shrink: 0;
        white-space: nowrap;
    }

    .inv-base {
        display: block;
        font-size: 0.7rem;
        color: #666;
        margin-top: 1px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .inv-stats {
        display: flex;
        gap: 8px;
        margin-top: 3px;
        font-size: 0.68rem;
        color: #9eb9d4;
    }

    .inv-mods {
        color: #555;
    }

    .inv-actions {
        display: flex;
        gap: 3px;
        flex-shrink: 0;
    }

    .btn-equip-inv {
        background: #2a3020;
        border: 1px solid #5a7a40;
        border-radius: 4px;
        color: #9ab870;
        cursor: pointer;
        font-size: 0.7rem;
        padding: 3px 8px;
        transition: background 0.15s;
    }

    .btn-equip-inv:hover:not(:disabled) {
        background: #3a4030;
    }

    .btn-equip-inv:disabled {
        opacity: 0.35;
        cursor: not-allowed;
    }

    .btn-remove-inv {
        background: #2a1212;
        border: 1px solid #5a3030;
        border-radius: 4px;
        color: #aa6666;
        cursor: pointer;
        font-size: 0.7rem;
        padding: 3px 6px;
        transition: background 0.15s;
    }

    .btn-remove-inv:hover {
        background: #3a1818;
    }

    .btn-danger {
        background: #2a1212;
        border: 1px solid #8a3a3a;
        border-radius: 4px;
        color: #c87070;
        cursor: pointer;
        font-size: 0.75rem;
        padding: 4px 10px;
    }

    .btn-danger:hover {
        background: #3a1818;
    }

    .status {
        color: #9eb9a4;
        font-size: 0.78rem;
        margin: 0;
    }

    /* Hover tooltip */
    :global(.item-tooltip) {
        position: fixed;
        z-index: 200;
        pointer-events: none;
        background: #1a1814;
        border: 1px solid #7a6a3e;
        border-radius: 6px;
        padding: 10px 14px;
        min-width: 180px;
        max-width: 280px;
        font-size: 0.78rem;
        color: #e0d6c2;
        box-shadow: 0 4px 18px rgba(0, 0, 0, 0.7);
    }

    :global(.item-tooltip .tt-name) {
        color: #c8a96e;
        font-size: 0.9rem;
        font-weight: 600;
        display: block;
        margin-bottom: 2px;
    }

    :global(.item-tooltip .tt-base) {
        color: #9b9b9b;
        font-size: 0.75rem;
        display: block;
        margin-bottom: 6px;
    }

    :global(.item-tooltip .tt-row) {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        margin-top: 2px;
    }

    :global(.item-tooltip .tt-label) {
        color: #777;
    }
    :global(.item-tooltip .tt-value) {
        color: #c8e0c8;
    }

    :global(.item-tooltip .tt-divider) {
        border: none;
        border-top: 1px solid #3a3428;
        margin: 6px 0;
    }

    :global(.item-tooltip .tt-class) {
        color: #6a9aca;
        font-size: 0.72rem;
    }

    :global(.item-tooltip .tt-variants) {
        color: #888;
        font-size: 0.72rem;
        margin-top: 4px;
    }

    /* ── Craft tab inner layout ─────────────────────────────────────────── */
    .craft-inner {
        display: flex;
        gap: 10px;
        flex: 1;
        overflow: hidden;
        min-height: 0;
    }

    .craft-browser {
        flex: 0 0 185px;
        background: #111115;
        border: 1px solid #2a2a30;
        border-radius: 6px;
        padding: 8px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
    }

    .craft-config {
        flex: 1;
        min-width: 0;
        background: #111115;
        border: 1px solid #2a2a30;
        border-radius: 6px;
        padding: 10px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
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
        transition:
            background 0.12s,
            color 0.12s;
    }
    .cat-tab:hover {
        background: #22222a;
        color: #c8b080;
    }
    .cat-tab.active {
        background: #22201a;
        border-color: #c8a96e;
        color: #c8a96e;
    }

    .subcat-list {
        display: flex;
        flex-direction: column;
        gap: 1px;
        max-height: 160px;
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
        transition:
            background 0.1s,
            color 0.1s;
    }
    .subcat-btn:hover {
        background: #1e1e26;
        color: #c8b080;
    }
    .subcat-btn.active {
        background: #22201a;
        color: #c8a96e;
        font-weight: 500;
    }

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
    .base-item {
        border-bottom: 1px solid #1a1a20;
    }
    .base-item.active .base-btn {
        background: #22201a;
        border-left: 2px solid #c8a96e;
    }
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
    .base-btn:hover {
        background: #1e1e26;
    }
    .base-name {
        color: #d0c8b0;
    }
    .base-lvl {
        color: #555;
        font-size: 0.65rem;
    }

    /* Craft config elements */
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
    .cfg-input.wide {
        width: 140px;
    }
    .cfg-input.narrow {
        width: 50px;
    }

    .overrides {
        margin-bottom: 8px;
    }
    .override-row {
        display: flex;
        align-items: center;
        gap: 5px;
        font-size: 0.73rem;
        margin-bottom: 3px;
        color: #888;
    }
    .ov-label {
        min-width: 80px;
        color: #888;
    }
    .ov-input {
        width: 55px;
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.75rem;
        padding: 2px 4px;
    }
    .ov-hint {
        color: #484840;
        font-size: 0.68rem;
    }

    .mod-section {
        margin-bottom: 8px;
    }
    .mod-section h4 {
        margin: 10px 0 5px;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: #9b8a5e;
        border-bottom: 1px solid #2a2a30;
        padding-bottom: 3px;
    }

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
    .filter-input:focus {
        border-color: #5a5a60;
    }

    .mod-list {
        max-height: 220px;
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
    .mod-row:last-child {
        border-bottom: none;
    }

    .mod-dn {
        flex: 1;
        font-size: 0.74rem;
        color: #b0a888;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .mod-row.active .mod-dn {
        color: #c8b888;
    }

    .tier-sel {
        flex: 0 0 auto;
        max-width: 155px;
        background: #1a1a22;
        border: 1px solid #3a3a48;
        border-radius: 3px;
        color: #9eb0c8;
        font-size: 0.7rem;
        padding: 2px 4px;
        cursor: pointer;
    }
    .tier-sel:disabled {
        opacity: 0.35;
        cursor: not-allowed;
    }
    .tier-sel:focus {
        border-color: #5a6a80;
        outline: none;
    }

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
        min-width: 90px;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .val-slider {
        flex: 1;
        max-width: 90px;
        accent-color: #6a90c8;
        cursor: pointer;
        height: 3px;
    }
    .val-num {
        width: 48px;
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
    .btn-add:hover:not(:disabled) {
        background: #4a3e1e;
    }
    .btn-add:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }
    .status.err {
        color: #c87070;
    }

    .hint.centered {
        text-align: center;
        margin-top: 40px;
    }

    /* ── Unique category browser ─────────────────────────────────────────── */
    .uniq-filters {
        display: flex;
        flex-direction: column;
        gap: 4px;
        margin-bottom: 6px;
    }
    .uniq-class-sel {
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.74rem;
        padding: 3px 5px;
        width: 100%;
    }
    .uniq-search {
        background: #1a1a1e;
        border: 1px solid #3a3a40;
        border-radius: 3px;
        color: #e0d6c2;
        font-size: 0.78rem;
        padding: 5px 8px;
        width: 100%;
        box-sizing: border-box;
        outline: none;
    }
    .uniq-search:focus {
        border-color: #c8a96e;
    }
    .uniq-list-btn {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: space-between;
    }
    .uniq-list-namecol {
        display: flex;
        flex-direction: column;
        gap: 1px;
        min-width: 0;
    }
    .uniq-list-name {
        color: #c8a96e;
        font-size: 0.78rem;
        font-weight: 500;
    }
    .uniq-list-base {
        color: #777;
        font-size: 0.68rem;
    }

    /* ── Unique roll picker ──────────────────────────────────────────────── */
    .unique-config {
        display: flex;
        flex-direction: column;
        gap: 0;
    }
    .unique-header {
        display: flex;
        align-items: baseline;
        gap: 8px;
        margin-bottom: 4px;
    }
    .unique-title {
        color: #c8a96e;
        font-size: 0.92rem;
        font-weight: 600;
    }
    .unique-base-lbl {
        color: #777;
        font-size: 0.78rem;
    }

    .uniq-line {
        margin-bottom: 5px;
    }
    .uniq-line-text {
        font-size: 0.78rem;
        color: #d0c0a0;
        padding: 2px 0;
        line-height: 1.4;
    }
    .uniq-line-text.line-header {
        color: #9b9b9b;
        font-style: italic;
    }
    .uniq-line.line-unmapped .uniq-line-text {
        color: #c83030;
    }
    .uniq-range-row {
        display: flex;
        align-items: center;
        gap: 5px;
        padding-left: 8px;
        margin-top: 2px;
    }

    /* ── PoE-style item tooltip in preview pane ──────────────────────────── */
    .poe-item-box {
        background: #0c0a08;
        border: 1px solid #8a7040;
        border-radius: 3px;
        padding: 10px 12px;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .poe-item-name {
        color: #c8a96e;
        font-size: 0.95rem;
        font-weight: 700;
        text-align: center;
        letter-spacing: 0.02em;
    }
    .poe-item-base {
        color: #c8b896;
        font-size: 0.78rem;
        text-align: center;
        margin-bottom: 2px;
    }
    .poe-inf-row {
        display: flex;
        gap: 5px;
        flex-wrap: wrap;
        margin: 3px 0;
    }
    .poe-inf-badge {
        background: #2a2018;
        border: 1px solid #8a7040;
        border-radius: 3px;
        color: #c8a040;
        font-size: 0.65rem;
        padding: 1px 6px;
    }
    .poe-league-lbl {
        color: #6a9aca;
        font-size: 0.7rem;
        text-align: center;
    }
    .poe-req {
        color: #8a8878;
        font-size: 0.72rem;
        margin-bottom: 2px;
    }
    .poe-stat {
        color: #c8b896;
        font-size: 0.75rem;
    }
    .poe-val {
        color: #7fbdff;
        font-weight: 500;
    }
    .poe-aps {
        color: #666;
    }
    .poe-mod {
        color: #d0c8a8;
        font-size: 0.75rem;
        line-height: 1.45;
    }
    .poe-mod.poe-mod-unmapped {
        color: #c03030;
    }
    .poe-unmapped-tag {
        display: inline-block;
        background: #3a0000;
        border: 1px solid #8a0000;
        border-radius: 2px;
        color: #c04040;
        font-size: 0.6rem;
        line-height: 1;
        padding: 0 3px;
        margin-left: 3px;
        vertical-align: middle;
        cursor: default;
    }
    .poe-line-header {
        color: #9b9b9b;
        font-size: 0.72rem;
        font-style: italic;
        margin: 2px 0;
    }
    /* Rarity variants */
    .poe-item-box.poe-item-magic {
        border-color: #6060b8;
    }
    .poe-item-box.poe-item-rare {
        border-color: #8a8020;
    }
    .poe-item-name.poe-name-magic {
        color: #8888ee;
    }
    .poe-item-name.poe-name-rare {
        color: #d4c832;
    }
    /* Mod source colours */
    .poe-mod.poe-mod-enchant {
        color: #b8aad0;
    }
    .poe-mod.poe-mod-crafted {
        color: #68a0d8;
    }
    .poe-mod.poe-mod-fractured {
        color: #a0c0d8;
    }
</style>
