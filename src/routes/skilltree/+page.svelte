<script lang="ts">
    import { onMount } from "svelte";
    import SkillTree from "../../components/SkillTree.svelte";
    import Sidebar from "../../components/Sidebar.svelte";
    import Header from "../../components/Header.svelte";
    import { commands } from "../../bindings";
    import { getBuildState } from "$lib/buildState.svelte";

    const build = getBuildState();

    onMount(async () => {
        if (!build.treeData) {
            const result = await commands.getTreeJson();
            if (result.status === "ok") {
                build.treeData = JSON.parse(result.data);
            }
        }
    });

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

    let selectedClass = $derived(classNameToIndex[build.characterClass] ?? 0);
</script>

<main class="layout">
    <Sidebar
        selectedCount={build.selectedCount}
        ascSelectedCount={build.ascSelectedCount}
        buildStats={build.buildStats}
    />
    <div class="tree-area">
        <div class="header-overlay">
            <Header
                bind:characterClass={build.characterClass}
                bind:ascendancy={build.ascendancy}
                bind:bloodline={build.bloodline}
                bind:level={build.level}
            />
        </div>
        <SkillTree
            treeData={build.treeData}
            bind:selectedCount={build.selectedCount}
            bind:ascSelectedCount={build.ascSelectedCount}
            bind:buildStats={build.buildStats}
            {selectedClass}
            selectedAscendancy={build.ascendancy}
            selectedBloodline={build.bloodline}
            selectedNodeIds={build.selectedNodeIds}
            selectedAscNodeIds={build.selectedAscNodeIds}
        />
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
