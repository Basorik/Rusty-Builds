<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { Application, Container, Graphics, Sprite } from "pixi.js";
    import { commands } from "../bindings";

    // --- Svelte 5: Props ---
    let {
        treeData,
        selectedCount = $bindable(0),
        selectedClass = 0,
    }: {
        treeData: any;
        selectedCount?: number;
        selectedClass?: number;
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
            const _class = selectedClass;
            processGraph(treeData);
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

    // --- Selection Logic ---
    function isConnectedToSelection(nodeId: number): boolean {
        const neighbors = adjacency.get(nodeId);
        if (!neighbors) return false;
        for (const nId of neighbors) {
            if (selectedNodeIds.has(nId)) return true;
        }
        return false;
    }

    function toggleNodeSelection(nodeId: number) {
        if (selectedNodeIds.has(nodeId)) {
            // Don't allow deselecting the class start node
            if (nodeId === classStartNodeId) return;
            // Don't allow deselecting if it would disconnect other selected nodes.
            // Temporarily remove the node, then BFS from the class start through
            // remaining selected nodes using the adjacency map. If any selected
            // node is unreachable, the removal would break connectivity — block it.
            if (!canDeselect(nodeId)) return;
            selectedNodeIds.delete(nodeId);
        } else {
            // Only allow selecting if connected to an already-selected node
            if (selectedNodeIds.size > 0 && !isConnectedToSelection(nodeId))
                return;
            selectedNodeIds.add(nodeId);
        }
        selectedCount = selectedNodeIds.size;
        drawSelection();
        syncSelectionToBackend();
    }

    function canDeselect(nodeId: number): boolean {
        // If removing this node leaves only the start node (or nothing), it's safe
        if (selectedNodeIds.size <= 2) return true;

        // BFS from classStartNodeId through selected nodes, skipping nodeId
        const visited = new Set<number>();
        const queue: number[] = [];

        if (classStartNodeId !== null && classStartNodeId !== nodeId) {
            queue.push(classStartNodeId);
            visited.add(classStartNodeId);
        }

        while (queue.length > 0) {
            const current = queue.shift()!;
            const neighbors = adjacency.get(current);
            if (!neighbors) continue;
            for (const neighbor of neighbors) {
                // Skip the node we're trying to remove, and only traverse selected nodes
                if (neighbor === nodeId) continue;
                if (!selectedNodeIds.has(neighbor)) continue;
                if (visited.has(neighbor)) continue;
                visited.add(neighbor);
                queue.push(neighbor);
            }
        }

        // Every selected node (except the one being removed) must be reachable
        for (const id of selectedNodeIds) {
            if (id === nodeId) continue;
            if (!visited.has(id)) return false;
        }
        return true;
    }

    function drawSelection() {
        if (!selectionGraphics) return;
        selectionGraphics.clear();
        if (selectedNodeIds.size === 0) return;

        // 1. Draw highlighted connections between two selected nodes
        selectionGraphics.setStrokeStyle({
            width: 24,
            color: 0x4488ff,
            alpha: 0.35,
        });
        for (const conn of renderConnections) {
            if (
                selectedNodeIds.has(conn.sourceId) &&
                selectedNodeIds.has(conn.targetId)
            ) {
                selectionGraphics.moveTo(conn.x1, conn.y1);
                selectionGraphics.lineTo(conn.x2, conn.y2);
            }
        }
        selectionGraphics.stroke();

        // 2. Outer glow ring
        selectionGraphics.setStrokeStyle({
            width: 14,
            color: 0x4488ff,
            alpha: 0.25,
        });
        for (const id of selectedNodeIds) {
            const node = nodeById.get(id);
            if (node) {
                selectionGraphics.circle(node.x, node.y, node.radius + 10);
            }
        }
        selectionGraphics.stroke();

        // 3. Bright inner ring
        selectionGraphics.setStrokeStyle({
            width: 5,
            color: 0x66aaff,
            alpha: 1,
        });
        for (const id of selectedNodeIds) {
            const node = nodeById.get(id);
            if (node) {
                selectionGraphics.circle(node.x, node.y, node.radius + 4);
            }
        }
        selectionGraphics.stroke();
    }

    let isSyncing = false;
    function syncSelectionToBackend() {
        if (syncTimer) clearTimeout(syncTimer);
        syncTimer = setTimeout(async () => {
            if (isSyncing) return; // Prevent overlapping calls
            try {
                isSyncing = true;
                const ids = Array.from(selectedNodeIds);
                const result = await commands.updateSelectedNodes(ids);
                // result is now the BuildStats object
                console.log("Received stats from Rust:", result);
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
            selectedCount = selectedNodeIds.size;
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
        if (app) app.renderer.resize(width, height);
        updateTransform();
    }

    function localX(e: MouseEvent): number {
        return e.clientX - containerDiv.getBoundingClientRect().left;
    }
    function localY(e: MouseEvent): number {
        return e.clientY - containerDiv.getBoundingClientRect().top;
    }

    function onMouseDown(e: MouseEvent) {
        isDragging = true;
        hasDragged = false;
        startX = localX(e) - transform.x;
        startY = localY(e) - transform.y;
        dragStartPos = { x: localX(e), y: localY(e) };
    }

    function onMouseMove(e: MouseEvent) {
        const mx = localX(e);
        const my = localY(e);
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
            tooltipPosition = { x: e.clientX, y: e.clientY };
        }
    }

    function onMouseUp(e: MouseEvent) {
        if (!hasDragged) {
            // Click — toggle node selection
            const mx = localX(e);
            const my = localY(e);
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
        const mx = localX(e);
        const my = localY(e);
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
