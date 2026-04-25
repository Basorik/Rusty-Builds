<script lang="ts">
    import Header from "../../components/Header.svelte";
    import Sidebar from "../../components/Sidebar.svelte";
    import SkillGroups from "../../components/SkillGroups.svelte";
    import { getBuildState } from "$lib/buildState.svelte";

    const build = getBuildState();
</script>

<main class="skills-page">
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
            <SkillGroups />
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

    .skills-page {
        display: flex;
        min-height: 100vh;
    }

    .page-body {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
    }

    .content {
        max-width: 1100px;
        margin: 0 auto;
        padding: 1.5rem;
        width: 100%;
        box-sizing: border-box;
    }
</style>
