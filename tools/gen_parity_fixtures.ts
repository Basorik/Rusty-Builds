#!/usr/bin/env bun
/**
 * tools/gen_parity_fixtures.ts
 *
 * Generates parity test fixtures by running PoB in headless mode.
 * For each XML in tests/fixtures/sources/ it:
 *   1. Appends pob_extract_tail.lua to PoB's HeadlessWrapper.lua
 *   2. Runs LuaJIT from the PoB src/ directory
 *   3. Captures the resulting JSON and writes tests/fixtures/<name>.json
 *
 * Usage:
 *   bun run tool:gen-fixtures                           # auto-detect PoB
 *   bun run tool:gen-fixtures --pob-path "C:\PoB"       # explicit PoB root
 *   bun run tool:gen-fixtures --pob-path "C:\PoB" my_build.xml other.xml
 *
 * When XML files are provided as arguments only those files are processed;
 * otherwise every *.xml in tests/fixtures/sources/ is used.
 *
 * Environment variable POB_PATH is also checked before the default locations.
 */

import { existsSync } from "fs";
import { readdir, readFile, writeFile, unlink, mkdir } from "fs/promises";
import { join, resolve, dirname, basename, extname } from "path";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";

// ── paths ─────────────────────────────────────────────────────────────────────
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = resolve(__dirname, "..");
const SOURCES_DIR = join(PROJECT_ROOT, "tests", "fixtures", "sources");
const FIXTURES_DIR = join(PROJECT_ROOT, "tests", "fixtures");
const TAIL_SCRIPT = join(__dirname, "pob_extract_tail.lua");

// ── PoB discovery ─────────────────────────────────────────────────────────────
const WINDOWS_DEFAULT_PATHS = [
    "C:\\Program Files (x86)\\Path of Building Community",
    "C:\\Program Files\\Path of Building Community",
    // Steam install location
    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Path of Building Community",
];

interface PoBEnv {
    /** Absolute path to LuaJIT executable */
    luajit: string;
    /** Absolute path to the PoB src/ directory (contains HeadlessWrapper.lua) */
    srcDir: string;
    /** Absolute path to the PoB runtime/ directory (contains Lua stdlib) */
    runtimeDir: string;
}

function findPoB(explicitRoot?: string): PoBEnv | null {
    const candidates: string[] = [];
    if (explicitRoot) candidates.push(explicitRoot);
    if (process.env.POB_PATH) candidates.push(process.env.POB_PATH);
    candidates.push(...WINDOWS_DEFAULT_PATHS);

    for (const root of candidates) {
        if (!root || !existsSync(root)) continue;

        // Windows: runtime/LuaJIT.exe  (official PoB installer)
        const win = join(root, "runtime", "LuaJIT.exe");
        // Some builds ship lua51.exe instead
        const win2 = join(root, "runtime", "lua51.exe");
        const src = join(root, "src");
        const rt = join(root, "runtime");

        if (existsSync(win) && existsSync(src)) return { luajit: win, srcDir: src, runtimeDir: rt };
        if (existsSync(win2) && existsSync(src)) return { luajit: win2, srcDir: src, runtimeDir: rt };
    }

    // Last resort: luajit on PATH (Linux/macOS manual install).
    // The user must also specify --pob-path for the src/ dir.
    const res = spawnSync("luajit", ["--version"], { encoding: "utf-8", shell: true });
    if (res.status === 0 && explicitRoot) {
        const src = join(explicitRoot, "src");
        const rt = join(explicitRoot, "runtime");
        if (existsSync(src)) {
            return { luajit: "luajit", srcDir: src, runtimeDir: rt };
        }
    }

    return null;
}

// ── arg parsing ───────────────────────────────────────────────────────────────
function parseArgs() {
    const args = process.argv.slice(2);
    let pobPath: string | undefined;
    const xmlFiles: string[] = [];

    for (let i = 0; i < args.length; i++) {
        if (args[i] === "--pob-path" && i + 1 < args.length) {
            pobPath = args[++i];
        } else if (args[i].endsWith(".xml")) {
            xmlFiles.push(resolve(args[i]));
        }
    }
    return { pobPath, xmlFiles };
}

// ── XML discovery ─────────────────────────────────────────────────────────────
async function findSourceXmls(explicitFiles: string[]): Promise<string[]> {
    if (explicitFiles.length > 0) return explicitFiles;

    if (!existsSync(SOURCES_DIR)) {
        console.error(`sources dir not found: ${SOURCES_DIR}`);
        console.error("Create it and add PoB XML exports to it, then re-run.");
        return [];
    }

    const entries = await readdir(SOURCES_DIR);
    return entries
        .filter(f => extname(f).toLowerCase() === ".xml")
        .map(f => join(SOURCES_DIR, f));
}

// ── combined script builder ───────────────────────────────────────────────────
async function buildCombinedScript(srcDir: string): Promise<string> {
    const wrapperPath = join(srcDir, "HeadlessWrapper.lua");
    if (!existsSync(wrapperPath)) {
        throw new Error(`HeadlessWrapper.lua not found at ${wrapperPath}`);
    }
    const wrapper = await readFile(wrapperPath, "utf-8");
    const tail = await readFile(TAIL_SCRIPT, "utf-8");
    return wrapper + "\n-- ===== RUSTY BUILDS EXTRACTION (appended by gen_parity_fixtures.ts) =====\n" + tail;
}

// ── runner ────────────────────────────────────────────────────────────────────
async function run() {
    const { pobPath, xmlFiles } = parseArgs();

    const pob = findPoB(pobPath);
    if (!pob) {
        console.error("Could not locate PoB installation.");
        console.error("Options:");
        console.error("  bun run tool:gen-fixtures --pob-path \"C:\\Path\\To\\PoB\"");
        console.error("  set POB_PATH=C:\\Path\\To\\PoB  (environment variable)");
        process.exit(1);
    }
    console.log(`Using PoB: ${pob.srcDir}`);
    console.log(`LuaJIT:    ${pob.luajit}`);

    const xmls = await findSourceXmls(xmlFiles);
    if (xmls.length === 0) {
        console.error("No XML source files found. Add PoB XML exports to:");
        console.error(`  ${SOURCES_DIR}`);
        process.exit(1);
    }
    console.log(`\nFound ${xmls.length} XML file(s):`);
    for (const x of xmls) console.log(`  ${x}`);

    // Ensure output directory exists
    await mkdir(FIXTURES_DIR, { recursive: true });

    // Write combined script into PoB src/ so relative require/dofile still works
    const combinedScript = await buildCombinedScript(pob.srcDir);
    const combinedPath = join(pob.srcDir, "_rusty_builds_extract.lua");
    await writeFile(combinedPath, combinedScript, "utf-8");

    // Build LUA_PATH — PoB runtime lua stdlib
    const luaPath = [
        join(pob.runtimeDir, "lua", "?.lua"),
        join(pob.runtimeDir, "lua", "?", "init.lua"),
        // PoB's own modules are loaded via LoadModule() not require(), but
        // some libs may use require() with local paths.
        "?.lua",
    ].join(process.platform === "win32" ? ";" : ":");

    console.log("\nRunning PoB headless...\n");

    const result = spawnSync(
        pob.luajit,
        [
            "_rusty_builds_extract.lua",
            ...xmls,
            "--out", FIXTURES_DIR,
        ],
        {
            cwd: pob.srcDir,
            encoding: "utf-8",
            stdio: "inherit",
            env: {
                ...process.env,
                LUA_PATH: luaPath,
            },
        }
    );

    // Clean up temp file even on failure
    await unlink(combinedPath).catch(() => { });

    if (result.error) {
        console.error("Failed to spawn LuaJIT:", result.error.message);
        process.exit(1);
    }
    if (result.status !== 0) {
        console.error(`\nLuaJIT exited with code ${result.status}`);
        process.exit(result.status ?? 1);
    }

    console.log("\nFixtures written to:", FIXTURES_DIR);
    console.log("Add #[test] entries in src-tauri/tests/pob_parity.rs for each new fixture.");
}

run().catch(err => {
    console.error("Unexpected error:", err);
    process.exit(1);
});
