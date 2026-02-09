<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { Application, Container, Graphics } from "pixi.js";

    export let treeData: any;

    let containerDiv: HTMLDivElement;
    let app: Application;
    let mainContainer: Container;
    let connectionGraphics: Graphics;
    let nodeGraphics: Graphics;
    let highlightGraphics: Graphics;

    let width = 800;
    let height = 600;
    let transform = { x: 0, y: 0, k: 0.1 }; // Start zoomed out
    let isDragging = false;
    let startX: number, startY: number;
    let hoveredNode: any = null;
    let tooltipPosition = { x: 0, y: 0 };

    // Processed data for rendering
    let renderNodes: any[] = [];
    let renderConnections: any[] = [];

    $: if (treeData && app) {
        processGraph(treeData);
    }

    function processGraph(data: any) {
        const { nodes, groups, constants } = data;
        const { orbitRadii, skillsPerOrbit } = constants;

        const tempNodes = new Map<string, any>();

        // 1. Calculate Absolute Node Positions
        for (const [nodeId, node] of Object.entries<any>(nodes)) {
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

            tempNodes.set(nodeId, { x, y, ...node, id: nodeId, radius: nodeRadius });
        }

        renderNodes = Array.from(tempNodes.values());

        // 2. Build Connections
        const tempConnections: any[] = [];
        for (const node of renderNodes) {
            if (node.out) {
                for (const targetId of node.out as string[]) {
                    const target = tempNodes.get(targetId);
                    if (target) {
                        tempConnections.push({
                            x1: node.x,
                            y1: node.y,
                            x2: target.x,
                            y2: target.y,
                        });
                    }
                }
            }
        }
        renderConnections = tempConnections;

        renderStaticGraph();
    }

    function renderStaticGraph() {
        if (!nodeGraphics || !connectionGraphics) return;

        // 1. Draw Connections (Batch all lines into one Graphics object)
        connectionGraphics.clear();
        // Pixi v8 syntax for stroke
        connectionGraphics.setStrokeStyle({ width: 20, color: 0x333333, alpha: 1 });
        
        for (const conn of renderConnections) {
            connectionGraphics.moveTo(conn.x1, conn.y1);
            connectionGraphics.lineTo(conn.x2, conn.y2);
        }
        connectionGraphics.stroke();

        // 2. Draw Nodes (Batch all circles into one Graphics object)
        nodeGraphics.clear();
        for (const node of renderNodes) {
            const color = node.isKeystone
                ? "#ff5555"
                : node.isNotable
                  ? "#ffcc00"
                  : "#555555";
            
            nodeGraphics.circle(node.x, node.y, node.radius);
            nodeGraphics.fill(color);
        }
    }

    function drawHighlight() {
        if (!highlightGraphics) return;
        highlightGraphics.clear();
        
        if (hoveredNode) {
            highlightGraphics.circle(hoveredNode.x, hoveredNode.y, hoveredNode.radius);
            highlightGraphics.fill(0xffffff);
        }
    }

    function updateTransform() {
        if (!mainContainer) return;
        mainContainer.position.set(transform.x + width / 2, transform.y + height / 2);
        mainContainer.scale.set(transform.k);
    }

    onMount(async () => {
        width = window.innerWidth;
        height = window.innerHeight;

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

        // Setup Scene Graph
        mainContainer = new Container();
        app.stage.addChild(mainContainer);

        connectionGraphics = new Graphics();
        nodeGraphics = new Graphics();
        highlightGraphics = new Graphics();

        // Order matters: Connections -> Nodes -> Highlight (Top)
        mainContainer.addChild(connectionGraphics);
        mainContainer.addChild(nodeGraphics);
        mainContainer.addChild(highlightGraphics);

        if (treeData) processGraph(treeData);
        updateTransform();

        window.addEventListener("resize", resize);
    });

    onDestroy(() => {
        if (app) {
            app.destroy(true, { children: true });
        }
        if (typeof window !== "undefined") {
            window.removeEventListener("resize", resize);
        }
    });

    function resize() {
        width = window.innerWidth;
        height = window.innerHeight;
        if (app) app.renderer.resize(width, height);
        updateTransform();
    }

    function onMouseDown(e: MouseEvent) {
        isDragging = true;
        startX = e.clientX - transform.x;
        startY = e.clientY - transform.y;
    }

    function onMouseMove(e: MouseEvent) {
        if (isDragging) {
            transform.x = e.clientX - startX;
            transform.y = e.clientY - startY;
            updateTransform();
        } else {
            // Hit detection
            const mouseX = e.clientX;
            const mouseY = e.clientY;

            // Convert screen coordinates to world coordinates
            const tX = transform.x + width / 2;
            const tY = transform.y + height / 2;
            const worldX = (mouseX - tX) / transform.k;
            const worldY = (mouseY - tY) / transform.k;

            const found = renderNodes.find((node) => {
                const dist = Math.hypot(node.x - worldX, node.y - worldY);
                return dist <= node.radius;
            });

            if (hoveredNode !== found) {
                hoveredNode = found || null;
                drawHighlight();
            }
            tooltipPosition = { x: mouseX, y: mouseY };
        }
    }

    function onMouseUp() {
        isDragging = false;
    }

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const scaleAmount = -e.deltaY * 0.001;
        // Limit min zoom to prevent flipping
        transform.k = Math.max(0.01, transform.k * (1 + scaleAmount));
        updateTransform();
    }
</script>

{#if hoveredNode}
    <div
        class="tooltip"
        style="top: {tooltipPosition.y + 15}px; left: {tooltipPosition.x + 15}px;"
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

<div
    bind:this={containerDiv}
    class="pixi-container"
    on:mousedown={onMouseDown}
    on:mousemove={onMouseMove}
    on:mouseup={onMouseUp}
    on:wheel={onWheel}
></div>

<style>
    .pixi-container {
        width: 100vw;
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
