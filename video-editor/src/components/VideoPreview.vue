<!-- src/components/VideoPreview.vue -->
<template>
    <section class="h-full flex flex-col">
        <!-- Hidden file input for video upload (always available) -->
        <input
            ref="fileInput"
            type="file"
            accept="video/*,image/*"
            class="hidden"
            @change="handleVideoUpload"
        />

        <!-- Video Area with Footer -->
        <div
            v-if="videoUrl"
            class="relative flex-1 overflow-hidden rounded-t-xl border border-white/10 bg-white/[0.03] shadow-[0_10px_40px_-10px_rgba(0,0,0,0.6)] flex flex-col"
        >
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

            <div class="relative flex-1 min-h-0">
                <canvas
                    ref="canvasElement"
                    class="block w-full h-full bg-black rounded-t-xl"
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
                            :data-start="it.start"
                            :data-end="it.end"
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

            <!-- Footer with playback controls and timeline -->
            <div
                class="flex-shrink-0 border-t border-white/10 bg-white/[0.02] px-4 py-2"
            >
                <!-- Row 1: Playback controls -->
                <div class="flex items-center justify-center gap-2 mb-2">
                    <button
                        @click="emit('toggle-play', !isPlaying)"
                        class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 transition"
                        :title="isPlaying ? 'Pause' : 'Play'"
                    >
                        <PauseIcon
                            v-if="isPlaying"
                            class="h-5 w-5"
                            aria-hidden="true"
                        />
                        <PlayIcon v-else class="h-5 w-5" aria-hidden="true" />
                    </button>
                    <button
                        @click="seekRelative(-5)"
                        class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 transition"
                        title="Rewind 5s"
                    >
                        <BackwardIcon class="h-5 w-5" aria-hidden="true" />
                    </button>
                    <button
                        @click="seekRelative(5)"
                        class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 transition"
                        title="Forward 5s"
                    >
                        <ForwardIcon class="h-5 w-5" aria-hidden="true" />
                    </button>
                    <button
                        @click="seekFrame(-1)"
                        class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 transition"
                        title="Previous frame"
                    >
                        <ChevronLeftIcon class="h-5 w-5" aria-hidden="true" />
                    </button>
                    <button
                        @click="seekFrame(1)"
                        class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 transition"
                        title="Next frame"
                    >
                        <ChevronRightIcon class="h-5 w-5" aria-hidden="true" />
                    </button>
                </div>

                <!-- Row 2: Timeline progress bar -->
                <div class="flex items-center gap-3">
                    <input
                        type="range"
                        min="0"
                        :max="duration || 0"
                        :value="currentTime || 0"
                        @input="handleSeek"
                        class="flex-1 appearance-none bg-white/10 h-2 rounded-full outline-none cursor-pointer"
                        aria-label="Timeline scrubber"
                    />
                    <span
                        class="text-[11px] font-mono text-neutral-400 w-20 text-right"
                        aria-live="polite"
                    >
                        {{ formatTime(currentTime || 0) }} /
                        {{ formatTime(duration || 0) }}
                    </span>
                </div>
            </div>
        </div>

        <div
            v-else
            class="flex-1 grid place-items-center rounded-b-xl border border-dashed border-white/10 bg-gradient-to-b from-white/[0.04] to-white/[0.02] min-h-0"
        >
            <div class="max-w-md space-y-4 px-4 text-center">
                <div
                    class="mx-auto grid size-16 place-items-center rounded-2xl bg-gradient-to-br from-violet-600/20 to-cyan-400/20 text-violet-300 backdrop-blur-sm"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-10 w-10"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="1.5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M9 10l4.553-2.276A1 1 0 0115 8.618v6.764a1 1 0 01-1.447.894L9 14M3 10l4.553-2.276A1 1 0 019 8.618v6.764a1 1 0 01-1.447.894L3 14"
                        />
                    </svg>
                </div>
                <div>
                    <p class="text-base font-medium text-neutral-200">
                        Upload a video to get started
                    </p>
                    <p class="text-sm text-neutral-400 mt-1">
                        MP4, WebM, MOV supported
                    </p>
                </div>
                <button
                    @click="openFilePicker"
                    class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-gradient-to-r from-violet-600 to-cyan-400 px-6 py-3 text-sm text-neutral-950 font-semibold shadow-lg hover:brightness-105 active:scale-95 transition mx-auto"
                >
                    <ArrowUpTrayIcon class="h-5 w-5" aria-hidden="true" />
                    Upload Video
                </button>
            </div>
        </div>
    </section>
</template>

<script setup>
import { ref, watch, onMounted, onBeforeUnmount, computed } from "vue";
import { VideoProcessor } from "../wasm/video_processor";
import { ArrowUpTrayIcon } from "@heroicons/vue/24/outline";
import {
    PlayIcon,
    PauseIcon,
    BackwardIcon,
    ForwardIcon,
    ChevronLeftIcon,
    ChevronRightIcon,
} from "@heroicons/vue/24/solid";

const props = defineProps({
    videoUrl: String,
    currentTime: Number,
    duration: Number,
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
    "upload-video",
    "seek",
    "toggle-play",
]);

const isPlaying = computed(() => {
    const v = videoElement.value;
    return v ? !v.paused && !v.ended : false;
});

const videoElement = ref(null);
const canvasElement = ref(null);
const overlayRoot = ref(null);
const fileInput = ref(null);
let ctx = null,
    processor = null;
let isLoopActive = false;
let width = 0,
    height = 0;

// Frame timing for performance monitoring
let lastFrameTime = 0;
const MIN_FRAME_TIME = 16; // ~60fps cap

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

/* Sync overlay video times with main video */
const syncOverlayVideos = () => {
    const t = props.currentTime || 0;
    if (!overlayRoot.value) return;
    const overlayVideos = overlayRoot.value.querySelectorAll("video");
    overlayVideos.forEach((video) => {
        const start = Number(video.dataset.start) || 0;
        const end = Number(video.dataset.end) || 0;
        // Only sync if the main video time is within the overlay's time range
        if (t >= start && t <= end) {
            const relativeTime = Math.max(0, t - start);
            if (video.currentTime !== relativeTime) {
                video.currentTime = relativeTime;
            }
        }
    });
};

/* Main draw loop with filter pipeline */
const loop = () => {
    const v = videoElement.value;
    const c = canvasElement.value;
    if (!v || !c || !ctx) {
        raf = requestAnimationFrame(loop);
        return;
    }

    // Only sync overlay videos if we have overlays to process
    if (props.overlayItems?.length) {
        syncOverlayVideos();
    }

    ctx.drawImage(v, 0, 0, c.width, c.height);

    try {
        // Skip WASM processing if no filters
        if (props.filters?.length) {
            const imageData = ctx.getImageData(0, 0, c.width, c.height);
            const processed = applyWASMFilter(imageData);
            ctx.putImageData(processed, 0, 0);
        }
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

// Footer control helpers
const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const mins = Math.floor(s / 60);
    const secs = Math.floor(s % 60);
    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
};
const handleSeek = (e) => emit("seek", parseFloat(e.target.value || "0"));
const seekRelative = (seconds) => {
    const v = videoElement.value;
    if (!v) return;
    const newTime = Math.max(
        0,
        Math.min(
            (v.currentTime || 0) + seconds,
            duration.value || v.duration || 0,
        ),
    );
    emit("seek", newTime);
};
const seekFrame = (direction) => {
    const v = videoElement.value;
    if (!v || !v.duration) return;
    // Assume ~30fps for frame stepping
    const frameTime = 1 / 30;
    const newTime = Math.max(
        0,
        Math.min((v.currentTime || 0) + frameTime * direction, v.duration),
    );
    emit("seek", newTime);
};

/* expose to parent */
const setCurrentTime = (t) => {
    const v = videoElement.value;
    if (v) v.currentTime = t;
};
const getVideoElement = () => videoElement.value;

const openFilePicker = () => {
    fileInput.value?.click();
};

const handleVideoUpload = (e) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;
    if (file.type.startsWith("video/") || file.type.startsWith("image/")) {
        const url = URL.createObjectURL(file);
        emit("upload-video", url);
    }
};

defineExpose({ setCurrentTime, getVideoElement });

onMounted(() => {
    /* loop started on metadata */
});
onBeforeUnmount(() => {
    isLoopActive = false;
    if (raf) cancelAnimationFrame(raf);
    raf = null;
    ctx = null;
    processor = null;
});
</script>
