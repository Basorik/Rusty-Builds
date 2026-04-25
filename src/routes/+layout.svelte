<script lang="ts">
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import LoadingOverlay from "../components/LoadingOverlay.svelte";

    let { children } = $props();

    let loadingStep = $state("Initializing…");
    let loadingFraction = $state(0);
    let isReady = $state(false);

    onMount(async () => {
        const unlisten = await listen<{
            step: string;
            fraction: number;
            done: boolean;
        }>("loading_progress", (event) => {
            loadingStep = event.payload.step;
            loadingFraction = event.payload.fraction;
            if (event.payload.done) {
                isReady = true;
                unlisten();
            }
        });
    });
</script>

{#if !isReady}
    <LoadingOverlay step={loadingStep} fraction={loadingFraction} />
{:else}
    {@render children()}
{/if}
