<script lang="ts">
    import SkillTree from "../../components/SkillTree.svelte";
    import Sidebar from "../../components/Sidebar.svelte";
    import Header from "../../components/Header.svelte";
    import treeData from "../../data.json";

    let selectedCount = $state(0);
    let ascSelectedCount = $state(0);

    // Class name from the Header dropdown
    let characterClass = $state("Marauder");
    let selectedAscendancy = $state("None");
    let selectedBloodline = $state("None");

    // Map class name → classStartIndex used by the tree data
    const classNameToIndex: Record<string, number> = {
        Scion: 0,
        Marauder: 1,
        Ranger: 2,
        Witch: 3,
        Duelist: 4,
        Templar: 5,
        Shadow: 6,
    };

    let selectedClass = $derived(classNameToIndex[characterClass] ?? 0);
</script>

<main class="layout">
    <Sidebar {selectedCount} {ascSelectedCount} />
    <div class="tree-area">
        <div class="header-overlay">
            <Header bind:characterClass bind:ascendancy={selectedAscendancy} bind:bloodline={selectedBloodline} />
        </div>
        <SkillTree {treeData} bind:selectedCount bind:ascSelectedCount {selectedClass} selectedAscendancy={selectedAscendancy} selectedBloodline={selectedBloodline} />
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
        overflow: hidden;
        background-color: #0a0a0a;
    }

    .layout {
        display: flex;
        width: 100vw;
        height: 100vh;
    }

    .tree-area {
        flex: 1;
        overflow: hidden;
        position: relative;
    }

    .header-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        z-index: 10;
    }
</style>
