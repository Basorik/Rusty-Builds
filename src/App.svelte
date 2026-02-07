<script>
    import SkillTree from "./components/SkillTree.svelte";
    import treeData from "./data.json";
</script>

<main>
    <SkillTree {treeData} />
</main>

<style>
    :global(body) {
        margin: 0;
        overflow: hidden;
        background-color: #0a0a0a;
    }
</style>
