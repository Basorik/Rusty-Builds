<script lang="ts">
    let {
        selectedCount = 0,
        characterClass = $bindable("Scion"),
    }: { selectedCount?: number; characterClass?: string } = $props();
    import { goto } from "$app/navigation";
    import { commands } from "../bindings";

    let level = $state(1);
    let ascendancy = $state("None");
    let bloodline = $state("None");

    const classData: Record<string, string[]> = {
        Scion: ["Ascendant"],
        Marauder: ["Juggernaut", "Berserker", "Chieftain"],
        Ranger: ["Raider", "Deadeye", "Pathfinder"],
        Duelist: ["Slayer", "Gladiator", "Champion"],
        Shadow: ["Assassin", "Saboteur", "Trickster"],
        Witch: ["Necromancer", "Occultist", "Elementalist"],
        Templar: ["Inquisitor", "Hierophant", "Guardian"],
    };

    const classes = Object.keys(classData);
    const bloodlines = [
        "None",
        "Crusader",
        "Redeemer",
        "Hunter",
        "Assassin",
        "Champion",
    ];

    let availableAscendancies = $derived([
        "None",
        ...(classData[characterClass] || []),
    ]);

    $effect(() => {
        if (!availableAscendancies.includes(ascendancy)) {
            ascendancy = "None";
        }
    });

    function Menu() {
        goto("/");
    }

    $effect(() => {
        if (availableAscendancies.includes(ascendancy)) {
            UpdateBuildInfo();
        }
    });

    async function UpdateBuildInfo() {
        const classArg = {
            class: characterClass,
            ascendancy: ascendancy === "None" ? null : ascendancy,
        };
        try {
            // @ts-ignore
            await commands.updateBuildInfo(level, classArg, bloodline);
        } catch (e) {
            console.error("Failed to update build info:", e);
        }
    }
</script>

<header class="ribbon">
    <div class="left">
        <button class="brand" onclick={Menu}>Rusty Builds</button>
    </div>

    <div class="selectors">
        <div class="selector">
            <label for="level">Level</label>
            <select id="level" bind:value={level}>
                {#each Array.from({ length: 100 }, (_, i) => i + 1) as l}
                    <option value={l}>{l}</option>
                {/each}
            </select>
        </div>

        <div class="selector">
            <label for="class">Class</label>
            <select id="class" bind:value={characterClass}>
                {#each classes as c}
                    <option value={c}>{c}</option>
                {/each}
            </select>
        </div>

        <div class="selector">
            <label for="ascendancy">Ascendancy</label>
            <select id="ascendancy" bind:value={ascendancy}>
                {#each availableAscendancies as a}
                    <option value={a}>{a}</option>
                {/each}
            </select>
        </div>

        <div class="selector">
            <label for="bloodline">Bloodline</label>
            <select id="bloodline" bind:value={bloodline}>
                {#each bloodlines as b}
                    <option value={b}>{b}</option>
                {/each}
            </select>
        </div>
    </div>
</header>

<style>
    .ribbon {
        display: flex;
        align-items: center;
        gap: 2rem;
        background-color: #141414;
        border-bottom: 1px solid #333;
        padding: 0 16px;
        height: 48px;
        box-sizing: border-box;
        font-family: sans-serif;
        pointer-events: auto;
    }

    .brand {
        background: none;
        border: none;
        color: #c8a95e;
        font-weight: bold;
        font-size: 1.1rem;
        cursor: pointer;
        padding: 0;
        white-space: nowrap;
    }

    .selectors {
        display: flex;
        gap: 24px;
        align-items: center;
    }

    .selector {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    label {
        color: #888;
        font-size: 0.8rem;
        text-transform: uppercase;
        white-space: nowrap;
    }

    select {
        background-color: #222;
        color: #ddd;
        border: 1px solid #444;
        border-radius: 4px;
        padding: 4px 8px;
        font-size: 0.9rem;
    }

    select:focus {
        outline: none;
        border-color: #c8a95e;
    }
</style>
