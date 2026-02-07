<script lang="ts">
    import { onMount } from "svelte";

    export let treeData: any;

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D | null;
    let width = 800;
    let height = 600;
    let transform = { x: 0, y: 0, k: 0.1 }; // Start zoomed out
    let isDragging = false;
    let startX: number, startY: number;

    // Processed data for rendering
    let renderNodes: any[] = [];
    let renderConnections: any[] = [];

    $: if (treeData) {
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

            tempNodes.set(nodeId, { x, y, ...node, id: nodeId });
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

        // Trigger initial draw
        if (ctx) draw();
    }

    function draw() {
        if (!ctx) return;

        // Clear and Reset
        ctx.clearRect(0, 0, width, height);
        ctx.save();

        // Apply Pan (translate) and Zoom (scale)
        // Center the view: transform.x/y + half screen width/height
        ctx.translate(transform.x + width / 2, transform.y + height / 2);
        ctx.scale(transform.k, transform.k);

        // Draw Connections
        ctx.beginPath();
        ctx.strokeStyle = "#333";
        ctx.lineWidth = 20;
        for (const conn of renderConnections) {
            ctx.moveTo(conn.x1, conn.y1);
            ctx.lineTo(conn.x2, conn.y2);
        }
        ctx.stroke();

        // Draw Nodes
        for (const node of renderNodes) {
            ctx.beginPath();
            // Size based on node type
            const radius = node.isKeystone ? 60 : node.isNotable ? 40 : 20;
            // Color based on node type
            ctx.fillStyle = node.isKeystone
                ? "#ff5555"
                : node.isNotable
                  ? "#ffcc00"
                  : "#555";

            ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI);
            ctx.fill();
        }

        ctx.restore();
    }

    onMount(() => {
        ctx = canvas.getContext("2d");
        resize();
        window.addEventListener("resize", resize);
        return () => window.removeEventListener("resize", resize);
    });

    function resize() {
        width = window.innerWidth;
        height = window.innerHeight;
        draw();
    }

    function onMouseDown(e: MouseEvent) {
        isDragging = true;
        startX = e.clientX - transform.x;
        startY = e.clientY - transform.y;
    }

    function onMouseMove(e: MouseEvent) {
        if (!isDragging) return;
        transform.x = e.clientX - startX;
        transform.y = e.clientY - startY;
        draw();
    }

    function onMouseUp() {
        isDragging = false;
    }

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const scaleAmount = -e.deltaY * 0.001;
        // Limit min zoom to prevent flipping
        transform.k = Math.max(0.01, transform.k * (1 + scaleAmount));
        draw();
    }
</script>

<canvas
    bind:this={canvas}
    {width}
    {height}
    on:mousedown={onMouseDown}
    on:mousemove={onMouseMove}
    on:mouseup={onMouseUp}
    on:wheel={onWheel}
    style="background: #0a0a0a; display: block;"
/>
