<!-- src/components/VideoPreview.vue -->
<template>
    <section>
        <div
            v-if="videoUrl"
            class="relative overflow-hidden rounded-xl border border-white/10 bg-white/[0.03] shadow-[0_10px_40px_-10px_rgba(0,0,0,0.6)]"
        >
            <!-- Hidden video; canvas shows the frame -->
            <video
                ref="videoElement"
                :src="videoUrl"
                crossorigin="anonymous"
                playsinline
                muted
                preload="auto"
                class="sr-only"
                @timeupdate="onTimeUpdate"
                @durationchange="onDurationChange"
                @loadedmetadata="onLoadedMetadata"
                @loadeddata="onLoadedData"
                @canplaythrough="onCanPlayThrough"
                @progress="onProgress"
                @play="emitPlaying"
                @pause="emitPlaying"
                @ended="emitPlaying"
            ></video>

            <div class="relative">
                <canvas
                    ref="canvasElement"
                    class="block w-full aspect-video bg-black"
                ></canvas>

                <!-- Overlays -->
                <div ref="overlayRoot" class="absolute inset-0">
                    <div
                        v-for="it in overlayItems"
                        :key="it.id"
                        class="group absolute"
                        :class="
                            isSelected(it) ? 'ring-2 ring-violet-400/70' : ''
                        "
                        :style="overlayStyle(it)"
                        @pointerdown.stop="startOverlayDrag(it, $event)"
                        @click.stop="selectOverlay(it)"
                    >
                        <!-- media -->
                        <video
                            v-if="it.kind === 'video' && it.src"
                            :src="it.src"
                            playsinline
                            muted
                            loop
                            class="h-full w-full object-contain rounded-md border border-white/10 pointer-events-none"
                        ></video>
                        <img
                            v-else-if="it.kind === 'image' && it.src"
                            :src="it.src"
                            class="h-full w-full object-contain rounded-md border border-white/10 pointer-events-none"
                        />
                        <!-- text -->
                        <div
                            v-else-if="it.kind === 'text'"
                            class="grid h-full w-full place-items-center rounded-md border border-white/10 pointer-events-none px-2"
                            :style="{
                                color: it.color || '#ffffff',
                                fontSize: (it.fontSize || 32) + 'px',
                                fontFamily:
                                    it.fontFamily ||
                                    'Inter, ui-sans-serif, system-ui',
                                fontWeight: it.fontWeight || 400,
                                fontStyle: it.fontStyle || 'normal',
                                textAlign: it.textAlign || 'center',
                                backgroundColor:
                                    it.backgroundColor || 'rgba(0,0,0,0.25)',
                            }"
                        >
                            {{ it.text || "Text" }}
                        </div>

                        <!-- resize handle (bottom-right) -->
                        <div
                            class="absolute bottom-0 right-0 m-0.5 hidden size-3 cursor-nwse-resize rounded-sm bg-white group-[.ring-2]:block"
                            @pointerdown.stop="startOverlayResize(it, $event)"
                        ></div>
                    </div>
                </div>

                <!-- Buffer overlay -->
                <div
                    v-if="showBuffer"
                    class="absolute inset-0 grid place-items-center bg-black/30 backdrop-blur-sm"
                >
                    <div
                        class="w-[320px] rounded-lg border border-white/10 bg-neutral-900/80 p-4 shadow-xl"
                    >
                        <div
                            class="mb-2 flex items-center justify-between text-xs text-neutral-300"
                        >
                            <span>Buffering</span>
                            <span>{{ Math.round(bufferedPercent) }}%</span>
                        </div>
                        <div
                            class="h-2 w-full overflow-hidden rounded-full bg-white/10"
                        >
                            <div
                                class="h-full rounded-full bg-gradient-to-r from-violet-500 to-cyan-400 transition-all"
                                :style="{ width: bufferedPercent + '%' }"
                            ></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div
            v-else
            class="grid place-items-center rounded-xl border border-dashed border-white/10 bg-white/[0.02] py-16 text-center"
        >
            <div class="max-w-md space-y-3 px-4">
                <div
                    class="mx-auto grid size-12 place-items-center rounded-lg bg-gradient-to-br from-violet-600 to-cyan-400 text-neutral-950 font-extrabold"
                >
                    V
                </div>
                <p class="text-sm text-neutral-300">
                    Upload a video to get started
                </p>
                <p class="text-xs text-neutral-500">MP4, WebM, MOV</p>
            </div>
        </div>
    </section>
</template>

<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from "vue";
import { VideoProcessor } from "../wasm/video_processor";

const props = defineProps({
    videoUrl: String,
    currentTime: Number,
    filters: { type: Array, default: () => [] },
    overlayItems: { type: Array, default: () => [] }, // [{id,trackId,kind:'video'|'image'|'text', x,y,w,h,z,text,color,fontSize,fontFamily,src}]
    selected: { type: Array, default: () => [] }, // [{trackId,itemId}]
});
const emit = defineEmits([
    "time-update",
    "duration-change",
    "ready",
    "playing-change",
    "overlay-move",
    "overlay-resize",
    "overlay-select",
]);

const videoElement = ref(null);
const canvasElement = ref(null);
const overlayRoot = ref(null);
let ctx = null,
    raf = null,
    processor = null;
let width = 0,
    height = 0;

// Buffering UI
const bufferedPercent = ref(0);
const showBuffer = ref(true);
const updateBufferUI = () => {
    const v = videoElement.value;
    if (!v) return;
    const dur = v.duration;
    if (!isFinite(dur) || dur <= 0) {
        bufferedPercent.value = v.readyState >= 3 ? 100 : 0;
        return;
    }
    const ranges = v.buffered;
    let end = 0;
    for (let i = 0; i < ranges.length; i++) end = Math.max(end, ranges.end(i));
    bufferedPercent.value = Math.min(100, Math.max(0, (end / dur) * 100));
    if (v.readyState >= 4) showBuffer.value = false;
};

const onProgress = () => updateBufferUI();
const onLoadedData = () => {
    updateBufferUI();
    emit("ready");
};
const onCanPlayThrough = () => {
    bufferedPercent.value = 100;
    showBuffer.value = false;
};

const onLoadedMetadata = () => {
    const v = videoElement.value;
    const c = canvasElement.value;
    if (!v || !c) return;
    c.width = v.videoWidth || 1280;
    c.height = v.videoHeight || 720;
    width = c.width;
    height = c.height;
    ctx = c.getContext("2d");
    ensureProcessor();
    emit("duration-change", v.duration || 0);
    loop();
};
const onTimeUpdate = () => {
    const v = videoElement.value;
    if (!v) return;
    emit("time-update", v.currentTime || 0);
    updateBufferUI();
};
const emitPlaying = () => {
    const v = videoElement.value;
    if (!v) return;
    emit("playing-change", !v.paused && !v.ended);
};

const ensureProcessor = () => {
    if (!processor && width > 0 && height > 0)
        processor = new VideoProcessor(width, height);
};

/* Frame processing with WASM filters */
const applyWASMFilter = (imageData) => {
    if (!processor || !props.filters.length) return imageData;

    try {
        const pixels = new Uint8Array(imageData.data);
        processor.load_frame(pixels);

        for (const f of props.filters) {
            if (f.name === "grayscale" && f.params?.intensity?.value > 0) {
                processor.apply_grayscale();
            } else if (f.name === "sepia" && f.params?.intensity?.value > 0) {
                processor.apply_sepia();
            } else if (f.name === "brightness") {
                const level = f.params?.level?.value ?? 1.0;
                processor.apply_brightness(level);
            } else if (f.name === "contrast") {
                const level = f.params?.level?.value ?? 1.0;
                processor.apply_contrast(level);
            } else if (f.name === "saturation") {
                const level = f.params?.level?.value ?? 1.0;
                processor.apply_saturation(level);
            } else if (f.name === "blur") {
                const radius = f.params?.radius?.value ?? 2;
                processor.apply_blur(radius);
            }
        }

        const processedPixels = processor.get_frame();
        imageData.data.set(processedPixels);
    } catch (e) {
        console.warn("WASM filter error:", e);
    }

    return imageData;
};

/* Main draw loop with filter pipeline */
const loop = () => {
    const v = videoElement.value;
    const c = canvasElement.value;
    if (!v || !c || !ctx) {
        raf = requestAnimationFrame(loop);
        return;
    }

    ctx.drawImage(v, 0, 0, c.width, c.height);

    try {
        const imageData = ctx.getImageData(0, 0, c.width, c.height);
        const processed = applyWASMFilter(imageData);
        ctx.putImageData(processed, 0, 0);
    } catch (e) {
        console.warn("Canvas filter processing failed:", e);
    }

    raf = requestAnimationFrame(loop);
};

/* ---------- Overlay geometry ---------- */
const clamp01 = (x) => Math.min(1, Math.max(0, x));
const overlayStyle = (it) => {
    // clamp so overlay never fully vanishes
    const x = clamp01(Math.min(it.x ?? 0, 1 - (it.w ?? 0.1)));
    const y = clamp01(Math.min(it.y ?? 0, 1 - (it.h ?? 0.1)));
    const w = clamp01(Math.max(it.w ?? 0.1, 0.02));
    const h = clamp01(Math.max(it.h ?? 0.1, 0.02));
    return {
        left: x * 100 + "%",
        top: y * 100 + "%",
        width: w * 100 + "%",
        height: h * 100 + "%",
        zIndex: String(it.z ?? 0),
    };
};
const textStyle = (it) => ({
    color: it.color || "#ffffff",
    fontSize: (it.fontSize || 24) + "px",
    fontFamily: it.fontFamily || "Inter, ui-sans-serif, system-ui",
    fontWeight: it.fontWeight || 400,
    fontStyle: it.fontStyle || "normal",
    textAlign: it.textAlign || "center",
    backgroundColor: it.backgroundColor || "rgba(0,0,0,0.25)",
    opacity: (it.opacity ?? 100) / 100,
});
const isSelected = (it) =>
    props.selected?.some((s) => s.itemId === it.id && s.trackId === it.trackId);

/* ---------- Overlay drag/resize ---------- */
let dragState = null;
let resizeState = null;

const selectOverlay = (it) => {
    emit("overlay-select", { id: it.id, trackId: it.trackId });
};

const startOverlayDrag = (it, e) => {
    // only drag with primary button
    if (e.button !== 0) return;
    const root = overlayRoot.value;
    const rect = root?.getBoundingClientRect();
    if (!rect) return;
    const pxToNormX = (px) => px / rect.width;
    const pxToNormY = (px) => px / rect.height;
    dragState = {
        id: it.id,
        trackId: it.trackId,
        startX: e.clientX,
        startY: e.clientY,
        baseX: it.x || 0,
        baseY: it.y || 0,
        w: it.w || 0.1,
        h: it.h || 0.1,
        pxToNormX,
        pxToNormY,
    };
    window.addEventListener("pointermove", onDragMove);
    window.addEventListener("pointerup", onDragEnd, { once: true });
};
const onDragMove = (e) => {
    if (!dragState) return;
    const dx = dragState.pxToNormX(e.clientX - dragState.startX);
    const dy = dragState.pxToNormY(e.clientY - dragState.startY);
    let nx = dragState.baseX + dx;
    let ny = dragState.baseY + dy;
    // keep inside bounds considering size
    nx = Math.min(Math.max(nx, 0), 1 - dragState.w);
    ny = Math.min(Math.max(ny, 0), 1 - dragState.h);
    emit("overlay-move", {
        id: dragState.id,
        trackId: dragState.trackId,
        x: nx,
        y: ny,
    });
};
const onDragEnd = () => {
    dragState = null;
    window.removeEventListener("pointermove", onDragMove);
};

const startOverlayResize = (it, e) => {
    if (e.button !== 0) return;
    const root = overlayRoot.value;
    const rect = root?.getBoundingClientRect();
    if (!rect) return;
    const pxToNormX = (px) => px / rect.width;
    const pxToNormY = (px) => px / rect.height;
    resizeState = {
        id: it.id,
        trackId: it.trackId,
        startX: e.clientX,
        startY: e.clientY,
        baseW: it.w || 0.1,
        baseH: it.h || 0.1,
        pxToNormX,
        pxToNormY,
    };
    window.addEventListener("pointermove", onResizeMove);
    window.addEventListener("pointerup", onResizeEnd, { once: true });
};
const onResizeMove = (e) => {
    if (!resizeState) return;
    const dw = resizeState.pxToNormX(e.clientX - resizeState.startX);
    const dh = resizeState.pxToNormY(e.clientY - resizeState.startY);
    let nw = Math.max(0.02, resizeState.baseW + dw);
    let nh = Math.max(0.02, resizeState.baseH + dh);
    nw = Math.min(nw, 1);
    nh = Math.min(nh, 1);
    emit("overlay-resize", {
        id: resizeState.id,
        trackId: resizeState.trackId,
        w: nw,
        h: nh,
    });
};
const onResizeEnd = () => {
    resizeState = null;
    window.removeEventListener("pointermove", onResizeMove);
};

/* expose to parent */
const setCurrentTime = (t) => {
    const v = videoElement.value;
    if (v) v.currentTime = t;
};
const getVideoElement = () => videoElement.value;
defineExpose({ setCurrentTime, getVideoElement });

onMounted(() => {
    /* loop started on metadata */
});
onBeforeUnmount(() => {
    if (raf) cancelAnimationFrame(raf);
});
</script>
