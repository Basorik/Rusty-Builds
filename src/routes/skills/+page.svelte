<script lang="ts">
    import Header from "../../components/Header.svelte";
    import SkillGroups from "../../components/SkillGroups.svelte";
    import { getBuildState } from "$lib/buildState.svelte";

    const build = getBuildState();
</script>

<main class="skills-page">
    <Header
        bind:characterClass={build.characterClass}
        bind:ascendancy={build.ascendancy}
        bind:bloodline={build.bloodline}
        bind:level={build.level}
    />

    <section class="content">
        <SkillGroups />
    </section>
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
        flex-direction: column;
        min-height: 100vh;
    }

    .content {
        max-width: 720px;
        margin: 0 auto;
        padding: 1.5rem;
        width: 100%;
        box-sizing: border-box;
    }
</style>
