/**
 * fetch_data.ts
 *
 * Downloads all development data files needed for Rusty Builds:
 *
 *   1. All top-level JSON files from repoe-fork PoB data export
 *   2. The official PoE 1 passive skill tree from grindinggear/skilltree-export
 *   3. The full RePoE data export (base_items, mods, gems, stats, etc.)
 *
 * Output directory: src-tauri/data/
 *
 * Usage:
 *   bun run tools/fetch_data.ts           # download everything
 *   bun run tools/fetch_data.ts --pob     # only repoe-fork PoB files
 *   bun run tools/fetch_data.ts --tree    # only skill tree
 *   bun run tools/fetch_data.ts --repoe   # only RePoE data export
 *   bun run tool:fetch-data               # package.json alias
 */

import { mkdirSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

// ── Configuration ───────────────────────────────────────────────────

const OUT_DIR = resolve("src-tauri/data");

const POB_BASE_URL = "https://repoe-fork.github.io/pob-data/poe1";

const REPOE_BASE_URL = "https://repoe-fork.github.io";

const SKILLTREE_API =
    "https://api.github.com/repos/grindinggear/skilltree-export";

// ── Arg parsing ─────────────────────────────────────────────────────

const args = process.argv.slice(2);
const onlyPob = args.includes("--pob");
const onlyTree = args.includes("--tree");
const onlyRepoe = args.includes("--repoe");
const runAll = !onlyPob && !onlyTree && !onlyRepoe;

// ── Helpers ─────────────────────────────────────────────────────────

function ensureDir(dir: string) {
    if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}

async function downloadFile(
    url: string,
    destPath: string,
    label: string,
): Promise<void> {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status} fetching ${url}`);

    const bytes = await res.arrayBuffer();
    await Bun.write(destPath, bytes);

    const sizeMB = (bytes.byteLength / 1_048_576).toFixed(2);
    console.log(`  ✅ ${label.padEnd(40)} ${sizeMB} MB`);
}

/**
 * Recursively crawl a repoe-fork directory index page and return all
 * relative file paths (excluding .min.json files).
 *
 * @param dirUrl   Full URL of the directory index page, e.g. ".../poe1/Bases/"
 * @param prefix   Accumulated path prefix, e.g. "Bases/" — prepended to each filename
 */
async function crawlPobDir(dirUrl: string, prefix: string = ""): Promise<string[]> {
    const res = await fetch(dirUrl);
    if (!res.ok) throw new Error(`Failed to fetch index ${dirUrl}: HTTP ${res.status}`);
    const html = await res.text();

    const results: string[] = [];

    // Directory links — href ends with "/"
    const dirMatches = [...html.matchAll(/href="(\.\/[^"]+\/)"/g)];
    for (const m of dirMatches) {
        const subDir = m[1].slice(2); // e.g. "Bases/"
        const subUrl = `${POB_BASE_URL}/${prefix}${subDir}`;
        const subFiles = await crawlPobDir(subUrl, `${prefix}${subDir}`);
        results.push(...subFiles);
    }

    // File links — href ends with ".json" (not a directory)
    const fileMatches = [...html.matchAll(/href="(\.\/[^"]+\.json)"/g)];
    for (const m of fileMatches) {
        const filename = m[1].slice(2); // strip "./"
        if (!filename.endsWith(".min.json")) {
            results.push(`${prefix}${filename}`);
        }
    }

    return results;
}

/** Get the download URL for data.json at the latest release tag. */
async function fetchSkillTreeUrl(): Promise<{ url: string; tag: string }> {
    const relRes = await fetch(`${SKILLTREE_API}/releases/latest`, {
        headers: { Accept: "application/vnd.github+json" },
    });
    if (!relRes.ok) throw new Error(`GitHub API error: HTTP ${relRes.status}`);
    const release = await relRes.json();
    const tag: string = release.tag_name;

    // Raw content URL for data.json at the release tag
    const url = `https://raw.githubusercontent.com/grindinggear/skilltree-export/${tag}/data.json`;
    return { url, tag };
}

// ── Tasks ────────────────────────────────────────────────────────────

async function downloadPobData() {
    console.log("\n📦 Fetching repoe-fork PoB data files…");
    const files = await crawlPobDir(`${POB_BASE_URL}/`);
    console.log(`   Found ${files.length} files to download`);

    const pobDir = join(OUT_DIR, "pob");
    ensureDir(pobDir);

    for (const filename of files) {
        const url = `${POB_BASE_URL}/${filename}`;
        const dest = join(pobDir, filename);
        // Ensure subdirectory exists (e.g. pob/Bases/, pob/Uniques/Special/)
        ensureDir(join(pobDir, filename.split("/").slice(0, -1).join("/")));
        await downloadFile(url, dest, filename);
    }
    console.log(`\n   Written to ${pobDir}/`);
}

async function downloadSkillTree() {
    console.log("\n🌲 Fetching PoE 1 passive skill tree…");
    const { url, tag } = await fetchSkillTreeUrl();
    console.log(`   Latest release: ${tag}`);

    const treeDir = join(OUT_DIR, "tree");
    const versionDir = join(treeDir, tag);
    ensureDir(versionDir);

    const destPath = join(versionDir, "data.json");
    await downloadFile(url, destPath, `data.json (${tag})`);
    console.log(`\n   Written to ${versionDir}/`);
    console.log(`   ℹ️  Update DEFAULT_TREE_VERSION in lib.rs to "${tag}" if this is a new version.`);
}

/**
 * Crawl a repoe-fork directory page and return all .json file names
 * (excluding .min.json). Only returns files, not subdirectories.
 */
async function crawlRepoeDir(dirUrl: string): Promise<string[]> {
    const res = await fetch(dirUrl);
    if (!res.ok) throw new Error(`Failed to fetch ${dirUrl}: HTTP ${res.status}`);
    const html = await res.text();

    const results: string[] = [];
    const fileMatches = [...html.matchAll(/href="(?:\.\/)?([^"]+\.json)"/g)];
    for (const m of fileMatches) {
        const filename = m[1];
        if (!filename.endsWith(".min.json")) {
            results.push(filename);
        }
    }
    return results;
}

async function downloadRepoeData() {
    console.log("\n📚 Fetching RePoE data export (poe1)…");

    const repoeDir = join(OUT_DIR, "repoe");
    ensureDir(repoeDir);

    // 1. Crawl the poe1.html index to get all top-level JSON files
    console.log("   Scanning poe1 index…");
    const indexRes = await fetch(`${REPOE_BASE_URL}/poe1.html`);
    if (!indexRes.ok) throw new Error(`Failed to fetch poe1.html: HTTP ${indexRes.status}`);
    const indexHtml = await indexRes.text();

    // Extract all .json hrefs (not .min.json), skip directories
    const topLevelFiles: string[] = [];
    const fileMatches = [...indexHtml.matchAll(/href="\.\/([^"]+\.json)"/g)];
    for (const m of fileMatches) {
        const filename = m[1];
        if (!filename.endsWith(".min.json")) {
            topLevelFiles.push(filename);
        }
    }

    console.log(`   Found ${topLevelFiles.length} top-level JSON files`);

    for (const filename of topLevelFiles) {
        const url = `${REPOE_BASE_URL}/${filename}`;
        const dest = join(repoeDir, filename);
        await downloadFile(url, dest, filename);
    }

    // 2. Download stat_translations/ subdirectory
    console.log("\n   Scanning stat_translations/…");
    const statTransDir = join(repoeDir, "stat_translations");
    ensureDir(statTransDir);

    const statTransFiles = await crawlRepoeDir(`${REPOE_BASE_URL}/stat_translations/`);
    console.log(`   Found ${statTransFiles.length} stat translation files`);

    for (const filename of statTransFiles) {
        const url = `${REPOE_BASE_URL}/stat_translations/${filename}`;
        const dest = join(statTransDir, filename);
        await downloadFile(url, dest, `stat_translations/${filename}`);
    }

    // 3. Download passive_skill_trees/ subdirectory
    console.log("\n   Scanning passive_skill_trees/…");
    const passiveTreeDir = join(repoeDir, "passive_skill_trees");
    ensureDir(passiveTreeDir);

    const passiveTreeFiles = await crawlRepoeDir(`${REPOE_BASE_URL}/passive_skill_trees/`);
    console.log(`   Found ${passiveTreeFiles.length} passive skill tree files`);

    for (const filename of passiveTreeFiles) {
        const url = `${REPOE_BASE_URL}/passive_skill_trees/${filename}`;
        const dest = join(passiveTreeDir, filename);
        await downloadFile(url, dest, `passive_skill_trees/${filename}`);
    }

    console.log(`\n   Written to ${repoeDir}/`);
}

// ── Main ─────────────────────────────────────────────────────────────

async function main() {
    console.log(`Output directory: ${OUT_DIR}`);
    ensureDir(OUT_DIR);

    if (runAll || onlyPob) await downloadPobData();
    if (runAll || onlyTree) await downloadSkillTree();
    if (runAll || onlyRepoe) await downloadRepoeData();

    console.log("\n🎉 Done.\n");
}

main().catch((err) => {
    console.error("\n❌ Fatal:", err.message ?? err);
    process.exit(1);
});
