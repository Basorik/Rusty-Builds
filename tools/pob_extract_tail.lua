-- pob_extract_tail.lua
-- This file is appended to PoB's HeadlessWrapper.lua by gen_parity_fixtures.ts.
-- It must NOT be run standalone — it relies on HeadlessWrapper having already
-- initialised PoB and defined: loadBuildFromXML(), runCallback(), build
--
-- Usage (handled by gen_parity_fixtures.ts):
--   luajit _combined.lua <xml1> [<xml2> ...] --out <output_dir>
--
-- Each XML path is a PoB export.  For every XML the script writes
--   <output_dir>/<stem>.json  with the fixture format expected by pob_parity.rs.

-- PoB class IDs (1-based, as stored in spec.curClassId).
-- spec.curClass (string) is tried first; this is the fallback.
local CLASS_ID_NAMES = {
    [1] = "Scion",
    [2] = "Marauder",
    [3] = "Ranger",
    [4] = "Witch",
    [5] = "Duelist",
    [6] = "Templar",
    [7] = "Shadow",
}

local function round(v)
    if type(v) ~= "number" then return 0 end
    return math.floor(v + 0.5)
end

local function read_file(path)
    local f, err = io.open(path, "r")
    if not f then return nil, err end
    local content = f:read("*a")
    f:close()
    return content
end

local function write_file(path, content)
    local f, err = io.open(path, "w")
    if not f then return err end
    f:write(content)
    f:close()
    return nil
end

-- Parse --out <dir> from arg, return output_dir and remaining xml paths.
local function parse_args()
    local xml_paths = {}
    local output_dir = "."
    local i = 1
    while i <= #arg do
        if arg[i] == "--out" then
            i = i + 1
            output_dir = arg[i] or "."
        else
            xml_paths[#xml_paths + 1] = arg[i]
        end
        i = i + 1
    end
    return output_dir, xml_paths
end

local function extract_stem(path)
    -- Strip directory and .xml extension
    return (path:match("([^/\\]+)%.xml$") or path):gsub("%.xml$", "")
end

-- Build a JSON array string from a sorted list of integers.
local function node_ids_json(ids)
    table.sort(ids)
    local parts = {}
    for _, id in ipairs(ids) do
        parts[#parts + 1] = tostring(id)
    end
    -- Wrap with line breaks if many nodes, otherwise inline
    if #parts > 20 then
        return "[\n    " .. table.concat(parts, ",\n    ") .. "\n  ]"
    end
    return "[" .. table.concat(parts, ", ") .. "]"
end

local function process(xml_path, output_dir)
    local xml, err = read_file(xml_path)
    if not xml then
        io.stderr:write("ERROR reading " .. xml_path .. ": " .. tostring(err) .. "\n")
        return false
    end

    -- Load and calculate
    local ok, load_err = pcall(loadBuildFromXML, xml)
    if not ok then
        io.stderr:write("ERROR loading " .. xml_path .. ": " .. tostring(load_err) .. "\n")
        return false
    end
    -- Extra frame to ensure all calcs have run
    runCallback("OnFrame")

    local output = build.calcsTab.mainOutput
    local spec   = build.spec

    -- Collect passive node IDs (excludes ascendancy start nodes if desired)
    local node_ids = {}
    for nodeId, _ in pairs(spec.allocNodes) do
        node_ids[#node_ids + 1] = nodeId
    end

    -- Class name — prefer the string form PoB exposes directly
    local class_name = (type(spec.curClass) == "string" and spec.curClass ~= "")
        and spec.curClass
        or CLASS_ID_NAMES[spec.curClassId]
        or ("UnknownClass" .. tostring(spec.curClassId))

    local fixture = string.format(
[[{
  "class": "%s",
  "level": %d,
  "node_ids": %s,
  "expected": {
    "life": %d,
    "mana": %d,
    "energy_shield": %d,
    "fire_resist": %d,
    "cold_resist": %d,
    "lightning_resist": %d,
    "chaos_resist": %d,
    "strength": %d,
    "dexterity": %d,
    "intelligence": %d
  }
}
]],
        class_name,
        build.level or 1,
        node_ids_json(node_ids),
        round(output.Life),
        round(output.Mana),
        round(output.EnergyShield),
        round(output.FireResist),
        round(output.ColdResist),
        round(output.LightningResist),
        round(output.ChaosResist),
        round(output.Str),
        round(output.Dex),
        round(output.Int)
    )

    local stem = extract_stem(xml_path)
    local out_path = output_dir .. "/" .. stem .. ".json"
    local write_err = write_file(out_path, fixture)
    if write_err then
        io.stderr:write("ERROR writing " .. out_path .. ": " .. tostring(write_err) .. "\n")
        return false
    end

    io.write(string.format("  [OK] %s -> %s\n", stem, out_path))
    return true
end

-- ── main ──────────────────────────────────────────────────────────────────────
local output_dir, xml_paths = parse_args()

if #xml_paths == 0 then
    io.stderr:write("pob_extract_tail: no XML files provided\n")
    io.stderr:write("Usage: gen_parity_fixtures.ts handles this automatically.\n")
else
    local ok_count, fail_count = 0, 0
    for _, path in ipairs(xml_paths) do
        io.write("Processing: " .. path .. "\n")
        io.flush()
        if process(path, output_dir) then
            ok_count = ok_count + 1
        else
            fail_count = fail_count + 1
        end
    end
    io.write(string.format("\nDone: %d succeeded, %d failed\n", ok_count, fail_count))
    if fail_count > 0 then
        os.exit(1)
    end
end
