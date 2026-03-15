# Development Tools

Scripts in this directory are **dev-only helpers** — they are not bundled into the
final application. Run them with Bun from the project root.

## Available Scripts

### `fetch_data.ts`

Downloads all data files needed for development into `src-tauri/data/`:

| Subfolder | Source | Contents |
|---|---|---|
| `src-tauri/data/pob/` | repoe-fork | All top-level JSON files (Gems, Bases, Mods, etc.) |
| `src-tauri/data/tree/` | grindinggear/skilltree-export (latest release) | `data.json` — passive skill tree |

```sh
# Download everything
bun run tool:fetch-data

# Only repoe-fork PoB files
bun run tool:fetch-pob

# Only the passive skill tree
bun run tool:fetch-tree

# Or run directly with flags
bun run tools/fetch_data.ts [--pob] [--tree]
```

## Adding New Tools

1. Create a new `.ts` file in this directory.
2. Add a convenience alias in the root `package.json` under `scripts`
   with the `tool:` prefix (e.g. `"tool:my-script": "bun run tools/my_script.ts"`).
3. Document it in this README.

> **Note:** The `tools/tsconfig.json` extends Bun types so scripts have access
> to `Bun`, `process`, `node:fs`, etc. without polluting the main app's tsconfig.
