import type { BuildStats } from "../bindings";

// Module-level runes persist across route navigation.
// Only destroyed when the user navigates back to "/" (main menu).

let characterClass = $state("Marauder");
let ascendancy = $state("None");
let bloodline = $state("None");
let level = $state(1);
let treeData = $state<any>(null);
let selectedCount = $state(0);
let ascSelectedCount = $state(0);
let buildStats = $state<BuildStats | null>(null);
let selectedNodeIds: Set<number> = new Set();
let selectedAscNodeIds: Set<number> = new Set();

export function getBuildState() {
    return {
        get characterClass() { return characterClass; },
        set characterClass(v: string) { characterClass = v; },

        get ascendancy() { return ascendancy; },
        set ascendancy(v: string) { ascendancy = v; },

        get bloodline() { return bloodline; },
        set bloodline(v: string) { bloodline = v; },

        get level() { return level; },
        set level(v: number) { level = v; },

        get treeData() { return treeData; },
        set treeData(v: any) { treeData = v; },

        get selectedCount() { return selectedCount; },
        set selectedCount(v: number) { selectedCount = v; },

        get ascSelectedCount() { return ascSelectedCount; },
        set ascSelectedCount(v: number) { ascSelectedCount = v; },

        get buildStats() { return buildStats; },
        set buildStats(v: BuildStats | null) { buildStats = v; },

        get selectedNodeIds() { return selectedNodeIds; },
        get selectedAscNodeIds() { return selectedAscNodeIds; },
    };
}

export function resetBuildState() {
    characterClass = "Marauder";
    ascendancy = "None";
    bloodline = "None";
    level = 1;
    treeData = null;
    selectedCount = 0;
    ascSelectedCount = 0;
    buildStats = null;
    selectedNodeIds = new Set();
    selectedAscNodeIds = new Set();
}
