<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { Application, Container, Graphics, Sprite } from "pixi.js";
    import { commands, type BuildStats } from "../bindings";

    // --- Svelte 5: Props ---
    let {
        treeData,
        selectedCount = $bindable(0),
        ascSelectedCount = $bindable(0),
        buildStats = $bindable<BuildStats | null>(null),
        selectedClass = 0,
        selectedAscendancy = "None",
        selectedBloodline = "None",
    }: {
        treeData: any;
        selectedCount?: number;
        ascSelectedCount?: number;
        buildStats?: BuildStats | null;
        selectedClass?: number;
        selectedAscendancy?: string;
        selectedBloodline?: string;
    } = $props();

    let containerDiv: HTMLDivElement;
    let app: Application;
    let mainContainer: Container;
    let connectionGraphics: Graphics;
    let nodeContainer: Container;
    let highlightGraphics: Graphics;
    let selectionGraphics: Graphics;

    let width = 800;
    let height = 600;
    let transform = { x: 0, y: 0, k: 0.1 };
    let isDragging = false;
    let startX: number, startY: number;
    let hasDragged = false;
    let dragStartPos = { x: 0, y: 0 };
    let hoveredNode: any = $state(null);
    let tooltipPosition = $state({ x: 0, y: 0 });

    // Processed data for rendering
    let renderNodes: any[] = [];
    let renderConnections: any[] = [];
    let nodeById: Map<number, any> = new Map();

    // --- Node Selection ---
    let selectedNodeIds: Set<number> = new Set();
    let syncTimer: ReturnType<typeof setTimeout> | null = null;

    // --- Adjacency map for connectivity checks ---
    let adjacency: Map<number, Set<number>> = new Map();
    let classStartNodeId: number | null = null;

    // --- Ascendency / Bloodline allocation ---
    const MAX_ASC_POINTS = 8;
    let selectedAscNodeIds: Set<number> = new Set();
    let ascStartNodeId: number | null = null;       // class ascendancy start
    let bloodlineStartNodeId: number | null = null;  // bloodline start
    let ascAdjacency: Map<number, Set<number>> = new Map();
    let ascNodeIds: Set<number> = new Set();  // all node IDs in the active ascendency+bloodline group

    // --- Spatial Grid for O(1) hit detection ---
    const GRID_CELL_SIZE = 300;
    let spatialGrid: Map<string, any[]> = new Map();

    // --- Sprite-based rendering ---
    let nodeSprites: { sprite: Sprite; node: any }[] = [];
    let circleTexture: any = null;

    // --- Svelte 5: Reactive effect ---
    let appReady = $state(false);

    $effect(() => {
        if (appReady && treeData) {
            // Re-process when treeData or selectedClass changes
            // const _class = selectedClass;
            // const _asc = selectedAscendancy;
            // const _bl = selectedBloodline;
            processGraph(treeData);
        }
    });

    $effect(() => {
        if (appReady) {
            const _class = selectedClass;
            syncSelectionToBackend();
        }
    });

    // When ascendancy or bloodline selection changes, rebuild the allowed set and reset points
    $effect(() => {
        if (appReady && treeData) {
            const _asc = selectedAscendancy;
            const _bl = selectedBloodline;
            rebuildAscendencyState();
        }
    });

    // --- Spatial Grid Helpers ---
    function getCellKey(x: number, y: number): string {
        return `${Math.floor(x / GRID_CELL_SIZE)},${Math.floor(y / GRID_CELL_SIZE)}`;
    }

    function buildSpatialGrid() {
        spatialGrid = new Map();
        for (const node of renderNodes) {
            const minCx = Math.floor((node.x - node.radius) / GRID_CELL_SIZE);
            const maxCx = Math.floor((node.x + node.radius) / GRID_CELL_SIZE);
            const minCy = Math.floor((node.y - node.radius) / GRID_CELL_SIZE);
            const maxCy = Math.floor((node.y + node.radius) / GRID_CELL_SIZE);
            for (let cx = minCx; cx <= maxCx; cx++) {
                for (let cy = minCy; cy <= maxCy; cy++) {
                    const key = `${cx},${cy}`;
                    if (!spatialGrid.has(key)) spatialGrid.set(key, []);
                    spatialGrid.get(key)!.push(node);
                }
            }
        }
    }

    function queryGrid(worldX: number, worldY: number): any | null {
        const key = getCellKey(worldX, worldY);
        const candidates = spatialGrid.get(key);
        if (!candidates) return null;
        for (const node of candidates) {
            const dx = node.x - worldX;
            const dy = node.y - worldY;
            if (dx * dx + dy * dy <= node.radius * node.radius) {
                return node;
            }
        }
        return null;
    }

    // --- Count Helpers ---
    function getAdjustedSelectedCount(): number {
        let count = selectedNodeIds.size;
        if (classStartNodeId !== null && selectedNodeIds.has(classStartNodeId)) {
            count--;
        }
        return Math.max(0, count);
    }

    function getAdjustedAscSelectedCount(): number {
        let count = selectedAscNodeIds.size;
        if (ascStartNodeId !== null && selectedAscNodeIds.has(ascStartNodeId)) {
            count--;
        }
        if (bloodlineStartNodeId !== null && selectedAscNodeIds.has(bloodlineStartNodeId)) {
            count--;
        }
        return Math.max(0, count);
    }

    // --- Ascendency State Rebuild ---
    function rebuildAscendencyState() {
        // Clear previous ascendency selections
        selectedAscNodeIds.clear();
        ascStartNodeId = null;
        bloodlineStartNodeId = null;
        ascNodeIds = new Set();
        ascAdjacency = new Map();

        // Single pass: collect nodes belonging to the selected ascendancy and/or bloodline
        const wantAsc = selectedAscendancy !== "None";
        const wantBl  = selectedBloodline  !== "None";
        if (wantAsc || wantBl) {
            for (const node of renderNodes) {
                if (wantAsc && node.ascendancyName === selectedAscendancy && !node.isBloodline) {
                    ascNodeIds.add(node.id);
                    if (node.isAscendancyStart) ascStartNodeId = node.id;
                } else if (wantBl && node.ascendancyName === selectedBloodline && node.isBloodline) {
                    ascNodeIds.add(node.id);
                    if (node.isAscendancyStart) bloodlineStartNodeId = node.id;
                }
            }
        }

        // Build ascendency adjacency from renderConnections restricted to ascNodeIds
        for (const conn of renderConnections) {
            if (ascNodeIds.has(conn.sourceId) && ascNodeIds.has(conn.targetId)) {
                if (!ascAdjacency.has(conn.sourceId)) ascAdjacency.set(conn.sourceId, new Set());
                if (!ascAdjacency.has(conn.targetId)) ascAdjacency.set(conn.targetId, new Set());
                ascAdjacency.get(conn.sourceId)!.add(conn.targetId);
                ascAdjacency.get(conn.targetId)!.add(conn.sourceId);
            }
        }

        // Auto-select the start nodes (they don't count against MAX unless we want them to)
        if (ascStartNodeId !== null) {
            selectedAscNodeIds.add(ascStartNodeId);
        }
        if (bloodlineStartNodeId !== null) {
            selectedAscNodeIds.add(bloodlineStartNodeId);
        }

        ascSelectedCount = getAdjustedAscSelectedCount();
        drawSelection();
    }

    // --- Shared Graph Helpers ---
    /** Returns true if nodeId has at least one neighbor in the selected set. */
    function hasSelectedNeighbor(
        nodeId: number,
        adj: Map<number, Set<number>>,
        selected: Set<number>,
    ): boolean {
        const neighbors = adj.get(nodeId);
        if (!neighbors) return false;
        for (const nId of neighbors) {
            if (selected.has(nId)) return true;
        }
        return false;
    }

    /**
     * BFS from startIds through selected nodes (skipping removeId).
     * Returns true only if every node in selected (except removeId) is reachable.
     */
    function isStillConnectedAfterRemoval(
        startIds: (number | null)[],
        selected: Set<number>,
        adj: Map<number, Set<number>>,
        removeId: number,
    ): boolean {
        if (selected.size <= 2) return true;

        const visited = new Set<number>();
        const queue: number[] = [];

        for (const sid of startIds) {
            if (sid !== null && sid !== removeId && !visited.has(sid)) {
                queue.push(sid);
                visited.add(sid);
            }
        }

        while (queue.length > 0) {
            const current = queue.shift()!;
            const neighbors = adj.get(current);
            if (!neighbors) continue;
            for (const neighbor of neighbors) {
                if (neighbor === removeId || !selected.has(neighbor) || visited.has(neighbor)) continue;
                visited.add(neighbor);
                queue.push(neighbor);
            }
        }

        for (const id of selected) {
            if (id !== removeId && !visited.has(id)) return false;
        }
        return true;
    }

    // --- Selection Logic ---
    function toggleNodeSelection(nodeId: number) {
        // Route ascendency/bloodline nodes to separate handler
        if (ascNodeIds.has(nodeId)) {
            toggleAscNodeSelection(nodeId);
            return;
        }

        if (selectedNodeIds.has(nodeId)) {
            if (nodeId === classStartNodeId) return;
            if (!isStillConnectedAfterRemoval([classStartNodeId], selectedNodeIds, adjacency, nodeId)) return;
            selectedNodeIds.delete(nodeId);
        } else {
            if (selectedNodeIds.size > 0 && !hasSelectedNeighbor(nodeId, adjacency, selectedNodeIds))
                return;
            selectedNodeIds.add(nodeId);
        }
        selectedCount = getAdjustedSelectedCount();
        drawSelection();
        syncSelectionToBackend();
    }

    // --- Ascendency Node Selection ---
    function toggleAscNodeSelection(nodeId: number) {
        if (selectedAscNodeIds.has(nodeId)) {
            if (nodeId === ascStartNodeId || nodeId === bloodlineStartNodeId) return;
            if (!isStillConnectedAfterRemoval([ascStartNodeId, bloodlineStartNodeId], selectedAscNodeIds, ascAdjacency, nodeId)) return;
            selectedAscNodeIds.delete(nodeId);
        } else {
            if (getAdjustedAscSelectedCount() >= MAX_ASC_POINTS) return;
            if (selectedAscNodeIds.size > 0 && !hasSelectedNeighbor(nodeId, ascAdjacency, selectedAscNodeIds)) return;
            selectedAscNodeIds.add(nodeId);
        }
        ascSelectedCount = getAdjustedAscSelectedCount();
        drawSelection();
        syncSelectionToBackend();
    }

    /** Draw connection highlights + glow + ring for a set of selected nodes. */
    function drawSelectionRings(
        selected: Set<number>,
        connColor: number,
        glowColor: number,
        ringColor: number,
    ) {
        if (selected.size === 0) return;

        // 1. Highlighted connections
        selectionGraphics.setStrokeStyle({ width: 24, color: connColor, alpha: 0.35 });
        for (const conn of renderConnections) {
            if (selected.has(conn.sourceId) && selected.has(conn.targetId)) {
                selectionGraphics.moveTo(conn.x1, conn.y1);
                selectionGraphics.lineTo(conn.x2, conn.y2);
            }
        }
        selectionGraphics.stroke();

        // 2. Outer glow ring
        selectionGraphics.setStrokeStyle({ width: 14, color: glowColor, alpha: 0.25 });
        for (const id of selected) {
            const node = nodeById.get(id);
            if (node) selectionGraphics.circle(node.x, node.y, node.radius + 10);
        }
        selectionGraphics.stroke();

        // 3. Bright inner ring
        selectionGraphics.setStrokeStyle({ width: 5, color: ringColor, alpha: 1 });
        for (const id of selected) {
            const node = nodeById.get(id);
            if (node) selectionGraphics.circle(node.x, node.y, node.radius + 4);
        }
        selectionGraphics.stroke();
    }

    function drawSelection() {
        if (!selectionGraphics) return;
        selectionGraphics.clear();
        drawSelectionRings(selectedNodeIds, 0x4488ff, 0x4488ff, 0x66aaff);
        drawSelectionRings(selectedAscNodeIds, 0xc8a95e, 0xc8a95e, 0xe0c070);
    }

    let isSyncing = false;
    function syncSelectionToBackend() {
        if (syncTimer) clearTimeout(syncTimer);
        syncTimer = setTimeout(async () => {
            if (isSyncing) return; // Prevent overlapping calls
            try {
                isSyncing = true;
                // Combine regular + ascendency selected nodes for the backend
                const ids = [
                    ...Array.from(selectedNodeIds),
                    ...Array.from(selectedAscNodeIds),
                ];
                const result = await commands.updateSelectedNodes(ids);
                if (result.status === "ok") {
                    buildStats = result.data;
                }
            } catch (e) {
                console.error("Failed to sync selection:", e);
            } finally {
                isSyncing = false;
            }
        }, 50);
    }

    function processGraph(data: any) {
        const { nodes, groups, constants } = data;
        const { orbitRadii, skillsPerOrbit } = constants;

        const tempNodes = new Map<number, any>();

        // 1. Calculate Absolute Node Positions
        for (const [nodeIdStr, node] of Object.entries<any>(nodes)) {
            const nodeId = parseInt(nodeIdStr, 10);
            if (isNaN(nodeId)) continue; // Skip "root" or other non-numeric keys

            if (node.group === undefined) continue; // Skip virtual nodes
            const group = groups[node.group];
            if (!group) continue;

            const orbit = node.orbit || 0;
            const orbitIndex = node.orbitIndex || 0;
            const radius = orbitRadii[orbit];
            const skillsInOrbit = skillsPerOrbit[orbit];

            // Calculate angle (0 is top/12 o'clock, moving clockwise)
            const angle = (2 * Math.PI * orbitIndex) / skillsInOrbit;

            // PoE Coordinate Math: x = r * sin(a), y = -r * cos(a)
            const offsetX = radius * Math.sin(angle);
            const offsetY = -radius * Math.cos(angle);

            const x = group.x + offsetX;
            const y = group.y + offsetY;

            const nodeRadius = node.isKeystone ? 60 : node.isNotable ? 40 : 20;

            tempNodes.set(nodeId, {
                x,
                y,
                ...node,
                id: nodeId,
                radius: nodeRadius,
            });
        }

        renderNodes = Array.from(tempNodes.values());

        // Store ID → node lookup for selection rendering
        nodeById = tempNodes;

        // 2. Build Connections — skip cross-boundary edges (ascendancy ↔ regular tree)
        const tempConnections: any[] = [];
        for (const node of renderNodes) {
            if (node.out) {
                for (const targetIdStr of node.out as string[]) {
                    const targetId = parseInt(targetIdStr, 10);
                    const target = tempNodes.get(targetId);
                    if (!target) continue;
                    // Skip connections that bridge ascendancy and regular tree
                    const srcAsc = !!node.ascendancyName;
                    const tgtAsc = !!target.ascendancyName;
                    if (srcAsc !== tgtAsc) continue;
                    tempConnections.push({
                        sourceId: node.id,
                        targetId: target.id,
                        x1: node.x,
                        y1: node.y,
                        x2: target.x,
                        y2: target.y,
                    });
                }
            }
        }
        renderConnections = tempConnections;

        // 3. Build adjacency map for connectivity checks
        adjacency = new Map();
        for (const conn of renderConnections) {
            if (!adjacency.has(conn.sourceId))
                adjacency.set(conn.sourceId, new Set());
            if (!adjacency.has(conn.targetId))
                adjacency.set(conn.targetId, new Set());
            adjacency.get(conn.sourceId)!.add(conn.targetId);
            adjacency.get(conn.targetId)!.add(conn.sourceId);
        }

        // 4. Auto-select class start node
        classStartNodeId = null;
        for (const node of renderNodes) {
            if (node.classStartIndex === selectedClass) {
                classStartNodeId = node.id;
                break;
            }
        }
        if (
            classStartNodeId !== null &&
            !selectedNodeIds.has(classStartNodeId)
        ) {
            selectedNodeIds.clear();
            selectedNodeIds.add(classStartNodeId);
            selectedCount = getAdjustedSelectedCount();
        }

        buildSpatialGrid();
        renderStaticGraph();
    }

    function renderStaticGraph() {
        if (!nodeContainer || !connectionGraphics || !circleTexture) return;

        // Clear previous sprites
        nodeContainer.removeChildren();
        nodeSprites = [];

        // 1. Draw Connections (batch all lines into one Graphics object)
        connectionGraphics.clear();
        connectionGraphics.setStrokeStyle({
            width: 20,
            color: 0x1a1a1a,
            alpha: 1,
        });

        for (const conn of renderConnections) {
            connectionGraphics.moveTo(conn.x1, conn.y1);
            connectionGraphics.lineTo(conn.x2, conn.y2);
        }
        connectionGraphics.stroke();

        // 2. Create Sprites for Nodes (shared texture, GPU-instanced)
        for (const node of renderNodes) {
            const sprite = new Sprite(circleTexture);
            sprite.anchor.set(0.5);
            sprite.position.set(node.x, node.y);
            sprite.width = node.radius * 2;
            sprite.height = node.radius * 2;
            sprite.tint = node.isKeystone
                ? 0x992222
                : node.isNotable
                  ? 0x997700
                  : 0x2a2a2a;
            nodeContainer.addChild(sprite);
            nodeSprites.push({ sprite, node });
        }

        drawSelection();
        cullNodes();
    }

    function drawHighlight() {
        if (!highlightGraphics) return;
        highlightGraphics.clear();

        if (hoveredNode) {
            highlightGraphics.circle(
                hoveredNode.x,
                hoveredNode.y,
                hoveredNode.radius,
            );
            highlightGraphics.fill(0xffffff);
        }
    }

    // --- Viewport Culling ---
    function cullNodes() {
        if (!mainContainer || nodeSprites.length === 0) return;

        const tX = transform.x + width / 2;
        const tY = transform.y + height / 2;

        const worldLeft = (0 - tX) / transform.k;
        const worldRight = (width - tX) / transform.k;
        const worldTop = (0 - tY) / transform.k;
        const worldBottom = (height - tY) / transform.k;

        const padding = 100;

        for (const { sprite, node } of nodeSprites) {
            sprite.visible =
                node.x + node.radius >= worldLeft - padding &&
                node.x - node.radius <= worldRight + padding &&
                node.y + node.radius >= worldTop - padding &&
                node.y - node.radius <= worldBottom + padding;
        }
    }

    function updateTransform() {
        if (!mainContainer) return;
        mainContainer.position.set(
            transform.x + width / 2,
            transform.y + height / 2,
        );
        mainContainer.scale.set(transform.k);
        cullNodes();
    }

    onMount(async () => {
        width = containerDiv.clientWidth;
        height = containerDiv.clientHeight;

        // Initialize Pixi Application
        app = new Application();
        await app.init({
            width,
            height,
            backgroundColor: 0x0a0a0a,
            antialias: true,
            resolution: window.devicePixelRatio || 1,
            autoDensity: true,
        });

        containerDiv.appendChild(app.canvas);

        // Generate shared circle texture for all node sprites
        const gfx = new Graphics();
        gfx.circle(0, 0, 32);
        gfx.fill(0xffffff);
        circleTexture = app.renderer.generateTexture(gfx);
        gfx.destroy();

        // Setup Scene Graph
        mainContainer = new Container();
        app.stage.addChild(mainContainer);

        connectionGraphics = new Graphics();
        nodeContainer = new Container();
        selectionGraphics = new Graphics();
        highlightGraphics = new Graphics();

        // Order: Connections -> Nodes -> Selection rings -> Hover highlight
        mainContainer.addChild(connectionGraphics);
        mainContainer.addChild(nodeContainer);
        mainContainer.addChild(selectionGraphics);
        mainContainer.addChild(highlightGraphics);

        updateTransform();

        // Signal readiness — triggers $effect to process treeData
        appReady = true;

        window.addEventListener("resize", resize);
    });

    onDestroy(() => {
        if (syncTimer) clearTimeout(syncTimer);
        if (circleTexture) circleTexture.destroy(true);
        if (app) {
            app.destroy(true, { children: true });
        }
        if (typeof window !== "undefined") {
            window.removeEventListener("resize", resize);
        }
    });

    function resize() {
        width = containerDiv.clientWidth;
        height = containerDiv.clientHeight;
        updateCachedRect();
        if (app) app.renderer.resize(width, height);
        updateTransform();
    }

    // Cache bounding rect — recomputed on resize, avoids repeated layout reflows
    let cachedRect: DOMRect | null = null;
    function updateCachedRect() {
        if (containerDiv) cachedRect = containerDiv.getBoundingClientRect();
    }
    function localCoords(e: MouseEvent): { mx: number; my: number } {
        if (!cachedRect) updateCachedRect();
        return { mx: e.clientX - cachedRect!.left, my: e.clientY - cachedRect!.top };
    }

    function onMouseDown(e: MouseEvent) {
        isDragging = true;
        hasDragged = false;
        const { mx, my } = localCoords(e);
        startX = mx - transform.x;
        startY = my - transform.y;
        dragStartPos = { x: mx, y: my };
    }

    function onMouseMove(e: MouseEvent) {
        const { mx, my } = localCoords(e);
        if (isDragging) {
            const dx = mx - dragStartPos.x;
            const dy = my - dragStartPos.y;
            if (dx * dx + dy * dy > 25) hasDragged = true;

            transform.x = mx - startX;
            transform.y = my - startY;
            updateTransform();
        } else {
            // Convert screen coordinates to world coordinates
            const tX = transform.x + width / 2;
            const tY = transform.y + height / 2;
            const worldX = (mx - tX) / transform.k;
            const worldY = (my - tY) / transform.k;

            // Spatial grid lookup — O(1) instead of O(N)
            const found = queryGrid(worldX, worldY);

            if (hoveredNode !== found) {
                hoveredNode = found || null;
                drawHighlight();
            }
            // Only update tooltip position when tooltip is visible
            if (hoveredNode) {
                tooltipPosition = { x: e.clientX, y: e.clientY };
            }
        }
    }

    function onMouseUp(e: MouseEvent) {
        if (!hasDragged) {
            const { mx, my } = localCoords(e);
            const tX = transform.x + width / 2;
            const tY = transform.y + height / 2;
            const worldX = (mx - tX) / transform.k;
            const worldY = (my - tY) / transform.k;
            const clicked = queryGrid(worldX, worldY);
            if (clicked) {
                toggleNodeSelection(clicked.id);
            }
        }
        isDragging = false;
    }

    function onWheel(e: WheelEvent) {
        e.preventDefault();

        const factor = Math.pow(1.1, -e.deltaY / 100);

        const newScale = Math.min(Math.max(transform.k * factor, 0.01), 2);

        // Zoom towards mouse pointer
        const { mx, my } = localCoords(e);
        const worldMouseX = (mx - (transform.x + width / 2)) / transform.k;
        const worldMouseY = (my - (transform.y + height / 2)) / transform.k;

        transform.x -= worldMouseX * newScale - worldMouseX * transform.k;
        transform.y -= worldMouseY * newScale - worldMouseY * transform.k;

        transform.k = newScale;
        updateTransform();
    }
</script>

{#if hoveredNode}
    <div
        class="tooltip"
        style="top: {tooltipPosition.y + 15}px; left: {tooltipPosition.x +
            15}px;"
    >
        <div class="tooltip-title">{hoveredNode.name || "Unknown Skill"}</div>
        {#if hoveredNode.stats}
            <div class="tooltip-stats">
                {#each hoveredNode.stats as stat}
                    <div>{stat}</div>
                {/each}
            </div>
        {/if}
        {#if hoveredNode.description}
            <div class="tooltip-desc">{hoveredNode.description}</div>
        {/if}
    </div>
{/if}

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    bind:this={containerDiv}
    class="pixi-container"
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onwheel={onWheel}
></div>

<style>
    .pixi-container {
        width: 100%;
        height: 100vh;
        overflow: hidden;
        background: #0a0a0a;
        display: block;
    }

    .tooltip {
        position: fixed;
        background: rgba(10, 10, 10, 0.95);
        border: 1px solid #a38d6d;
        color: #dfcf99;
        padding: 10px;
        pointer-events: none;
        z-index: 100;
        border-radius: 4px;
        font-family: sans-serif;
        max-width: 300px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.5);
    }
    .tooltip-title {
        font-weight: bold;
        font-size: 1.1em;
        margin-bottom: 5px;
        color: #fff;
    }
    .tooltip-stats {
        color: #8888ff;
        font-size: 0.9em;
    }
    .tooltip-desc {
        margin-top: 5px;
        font-style: italic;
        color: #aaa;
        font-size: 0.85em;
    }
</style>
