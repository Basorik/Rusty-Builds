<script lang="ts">
    let {
        selectedCount = 0,
        characterClass = $bindable("Scion"),
        ascendancy = $bindable("None"),
        bloodline = $bindable("None"),
        level = $bindable(1),
    }: {
        selectedCount?: number;
        characterClass?: string;
        ascendancy?: string;
        bloodline?: string;
        level?: number;
    } = $props();
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { commands } from "../bindings";
    import { resetBuildState } from "$lib/buildState.svelte";

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
        "Aul",
        "Breachlord",
        "Catarina",
        "Delirious",
        "Farrul",
        "KingInTheMists",
        "Lycia",
        "Olroth",
        "Oshabi",
        "Primalist",
        "Trialmaster",
        "Warden",
        "Warlock",
    ];

    let availableAscendancies = $derived([
        "None",
        ...(classData[characterClass] || []),
    ]);

    // Track previous values to avoid re-sending on mount
    let prevClass = characterClass;
    let prevAsc = ascendancy;
    let prevBlood = bloodline;
    let prevLevel = level;

    $effect(() => {
        if (!availableAscendancies.includes(ascendancy)) {
            ascendancy = "None";
        }

        const changed =
            characterClass !== prevClass ||
            ascendancy !== prevAsc ||
            bloodline !== prevBlood ||
            level !== prevLevel;

        if (changed) {
            prevClass = characterClass;
            prevAsc = ascendancy;
            prevBlood = bloodline;
            prevLevel = level;
            UpdateBuildInfo();
        }
    });

    function Menu() {
        resetBuildState();
        goto("/");
    }

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
        <nav class="tabs">
            <button
                class="tab"
                class:active={$page.url.pathname === "/skilltree"}
                onclick={() => goto("/skilltree")}>Tree</button
            >
            <button
                class="tab"
                class:active={$page.url.pathname === "/skills"}
                onclick={() => goto("/skills")}>Skills</button
            >
            <button class="tab disabled" disabled title="Coming soon"
                >Items</button
            >
            <button class="tab disabled" disabled title="Coming soon"
                >Config</button
            >
            <button class="tab disabled" disabled title="Coming soon"
                >Calcs</button
            >
            <button
                class="tab"
                class:active={$page.url.pathname === "/debug"}
                onclick={() => goto("/debug")}>Debug</button
            >
        </nav>
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

    .left {
        display: flex;
        align-items: center;
        gap: 12px;
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

    .tabs {
        display: flex;
        gap: 2px;
    }

    .tab {
        background: none;
        border: none;
        border-bottom: 2px solid transparent;
        color: #8a8578;
        font-size: 0.82rem;
        cursor: pointer;
        padding: 6px 12px;
        transition:
            color 0.15s,
            border-color 0.15s;
    }

    .tab:hover:not(.disabled) {
        color: #e0d6c2;
    }

    .tab.active {
        color: #c8a95e;
        border-bottom-color: #c8a95e;
    }

    .tab.disabled {
        color: #4a4540;
        cursor: default;
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
