/**
 * fetch_data.ts
 *
 * Downloads all development data files needed for Rusty Builds:
 *
 *   1. All top-level JSON files from repoe-fork (PoB data export)
 *   2. The official PoE 1 passive skill tree from grindinggear/skilltree-export
 *
 * Output directory: src-tauri/data/
 *
 * Usage:
 *   bun run tools/fetch_data.ts          # download everything
 *   bun run tools/fetch_data.ts --pob    # only repoe-fork files
 *   bun run tools/fetch_data.ts --tree   # only skill tree
 *   bun run tool:fetch-data              # package.json alias
 */

import { mkdirSync, existsSync, symlinkSync, unlinkSync, lstatSync } from "node:fs";
import { join, resolve } from "node:path";

// ── Configuration ───────────────────────────────────────────────────

const OUT_DIR = resolve("src-tauri/data");

const POB_BASE_URL = "https://repoe-fork.github.io/pob-data/poe1";

const SKILLTREE_API =
    "https://api.github.com/repos/grindinggear/skilltree-export";

// ── Arg parsing ─────────────────────────────────────────────────────

const args = process.argv.slice(2);
const onlyPob = args.includes("--pob");
const onlyTree = args.includes("--tree");
const runAll = !onlyPob && !onlyTree;

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

/** Update the `tree/active.json` symlink to point at the given versioned file. */
function setActiveTree(treeDir: string, tag: string) {
    const activePath = join(treeDir, "active.json");
    const target = join(tag, "data.json"); // relative symlink

    try {
        lstatSync(activePath); // throws if missing
        unlinkSync(activePath);
    } catch {
        // didn't exist — that's fine
    }

    symlinkSync(target, activePath);
    console.log(`   🔗 active.json → ${target}`);
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
    setActiveTree(treeDir, tag);
    console.log(`\n   Written to ${versionDir}/`);
}

// ── Main ─────────────────────────────────────────────────────────────

async function main() {
    console.log(`Output directory: ${OUT_DIR}`);
    ensureDir(OUT_DIR);

    if (runAll || onlyPob) await downloadPobData();
    if (runAll || onlyTree) await downloadSkillTree();

    console.log("\n🎉 Done.\n");
}

main().catch((err) => {
    console.error("\n❌ Fatal:", err.message ?? err);
    process.exit(1);
});
