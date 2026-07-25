<!-- src/components/VideoEditor.vue -->
<template>
    <div class="video-editor space-y-4">
        <div class="grid grid-cols-12 gap-4">
            <!-- Preview + Overlays -->
            <div class="col-span-12 xl:col-span-8 space-y-3">
                <div
                    class="relative rounded-xl border border-white/10 bg-white/[0.03] p-3"
                >
                    <VideoPreview
                        ref="preview"
                        :video-url="videoUrl"
                        :current-time="currentTime"
                        :filters="filters"
                        :overlay-items="visibleOverlayItems"
                        :selected="selection"
                        @time-update="handleTimeUpdate"
                        @duration-change="handleDurationChange"
                        @ready="handleReady"
                        @playing-change="isPlaying = $event"
                        @overlay-move="onOverlayMove"
                        @overlay-resize="onOverlayResize"
                        @overlay-select="onOverlaySelect"
                    />
                </div>

                <!-- Uploader -->
                <div
                    class="upload-area"
                    @dragover.prevent
                    @drop="handleFileDrop"
                >
                    <div
                        v-if="!videoUrl"
                        class="rounded-xl border border-dashed border-white/10 bg-white/[0.02] p-6 text-center"
                    >
                        <input
                            type="file"
                            id="video-upload"
                            accept="video/*"
                            @change="handleFileUpload"
                            hidden
                        />
                        <label
                            for="video-upload"
                            class="inline-flex cursor-pointer items-center gap-2 rounded-md border border-white/10 bg-white/5 px-4 py-2 text-sm text-neutral-200 hover:bg-white/10 transition"
                        >
                            Upload a video
                        </label>
                        <p class="mt-2 text-xs text-neutral-500">
                            Supported: MP4, WebM, MOV
                        </p>
                    </div>
                </div>
            </div>

            <!-- Controls + Inspector -->
            <div class="col-span-12 xl:col-span-4 space-y-4">
                <Controls
                    :playing="isPlaying"
                    :filters="filters"
                    :duration="duration"
                    :current-time="currentTime"
                    @add-filter="addFilter"
                    @remove-filter="removeFilter"
                    @update-filter="updateFilter"
                    @seek="seekToTime"
                    @export="exportVideo"
                    @toggle-play="onTogglePlay"
                />

                <Inspector
                    :selected="selection"
                    :tracks="tracks"
                    :duration="duration"
                    @update-item="onUpdateItem"
                    @remove="onRemoveSelection"
                    @reset-item="onResetItem"
                />

                <MediaLibrary @remove="removeMediaFromLibrary" />
            </div>

            <!-- Timeline -->
            <div class="col-span-12">
                <Timeline
                    :duration="duration"
                    :current-time="currentTime"
                    :tracks="tracks"
                    :selected="selection"
                    @seek="seekToTime"
                    @add-clip="onAddClip"
                    @add-text="onAddText"
                    @add-track="onAddTrack"
                    @remove-item="onRemoveItem"
                    @split="onSplit"
                    @merge="onMerge"
                    @cut="onCut"
                    @selection-change="onSelectionChange"
                    @update-item="onUpdateItem"
                    @move-item="onMoveItem"
                />
            </div>
        </div>

        <!-- Export overlay -->
        <div
            v-if="exporting"
            class="fixed inset-0 z-50 grid place-items-center bg-black/50 backdrop-blur-sm"
        >
            <div
                class="w-[320px] rounded-lg border border-white/10 bg-neutral-900/90 p-5 shadow-xl"
            >
                <div
                    class="mb-3 flex items-center justify-between text-xs text-neutral-300"
                >
                    <span>Exporting</span>
                    <span>{{ exportProgress }}%</span>
                </div>
                <div
                    class="h-2 w-full overflow-hidden rounded-full bg-white/10"
                >
                    <div
                        class="h-full rounded-full bg-gradient-to-r from-violet-500 to-cyan-400 transition-all"
                        :style="{ width: exportProgress + '%' }"
                    ></div>
                </div>
            </div>
        </div>

        <!-- hidden pickers -->
        <input
            ref="clipPicker"
            type="file"
            accept="video/*,image/*"
            class="hidden"
            @change="onPickClip"
        />
    </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { useEditorStore } from "../stores/editor";
import Controls from "./Controls.vue";
import VideoPreview from "./VideoPreview.vue";
import Timeline from "./Timeline.vue";
import Inspector from "./Inspector.vue";
import MediaLibrary from "./MediaLibrary.vue";
import initWasm, {
    VideoProcessor,
    export_video,
} from "../wasm/video_processor";

// Use the centralized store
const store = useEditorStore();

// Media library handler
const removeMediaFromLibrary = (id) => {
    store.removeMedia(id);
};

// Local refs for component state
const videoUrl = ref(null);
const currentTime = ref(0);
const duration = ref(0);
const isPlaying = ref(false);
const filters = ref([]);
const tracks = ref([
    { id: "t-video-1", type: "video", label: "Video Track", items: [] },
    { id: "t-audio-1", type: "audio", label: "Audio Track", items: [] },
    { id: "t-text-1", type: "text", label: "Text Track", items: [] },
]);
const selection = ref([]);

// Other component refs
const preview = ref(null);
const exporting = ref(false);
const exportProgress = ref(0);
let videoProcessor = null;
const clipPicker = ref(null);
let pendingClipTrackId = null;

/* ---------- wasm boot ---------- */
onMounted(async () => {
    try {
        await initWasm();
    } catch (e) {
        console.error("WASM load failed", e);
    }
});

/* ---------- preview derived overlays ---------- */
const visibleOverlayItems = computed(() => {
    const t = currentTime.value;
    const out = [];
    for (const tr of tracks.value) {
        if (tr.type === "audio") continue;
        for (const it of tr.items) {
            if (t >= it.start && t <= it.end) {
                out.push({ ...it, trackId: tr.id });
            }
        }
    }
    return out.sort((a, b) => (a.z || 0) - (b.z || 0));
});

/* ---------- file handlers ---------- */
const handleFileUpload = (e) => {
    const file = e.target.files?.[0];
    if (file && file.type.startsWith("video/")) {
        videoUrl.value = URL.createObjectURL(file);
        filters.value = [];
        currentTime.value = 0;
        isPlaying.value = false;
    }
};
const handleFileDrop = (e) => {
    const file = e.dataTransfer?.files?.[0];
    if (file && file.type.startsWith("video/")) {
        videoUrl.value = URL.createObjectURL(file);
        filters.value = [];
        currentTime.value = 0;
        isPlaying.value = false;
    }
};

/* ---------- preview events ---------- */
const handleTimeUpdate = (t) => {
    currentTime.value = t;
};
const handleDurationChange = (d) => {
    duration.value = d;
    ensureBaseClips();
};
const handleReady = () => {};

/* ---------- base clips for main source ---------- */
const ensureBaseClips = () => {
    const d = duration.value || 0;
    if (!d) return;
    const vTrack = tracks.value.find((t) => t.type === "video");
    const aTrack = tracks.value.find((t) => t.type === "audio");
    if (vTrack && vTrack.items.every((it) => it.kind !== "source-video")) {
        vTrack.items.unshift({
            id: "source-video",
            kind: "source-video",
            label: "Source Video",
            start: 0,
            end: d,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
            z: -1000,
        });
    }
    if (aTrack && aTrack.items.every((it) => it.kind !== "source-audio")) {
        aTrack.items.unshift({
            id: "source-audio",
            kind: "source-audio",
            label: "Source Audio",
            start: 0,
            end: d,
        });
    }
};

/* ---------- transport ---------- */
const seekToTime = (t) => {
    preview.value?.setCurrentTime?.(t);
};
const onTogglePlay = async (shouldPlay) => {
    const videoEl = preview.value?.getVideoElement?.();
    if (!videoEl) return;
    try {
        if (shouldPlay) {
            await videoEl.play();
            isPlaying.value = true;
        } else {
            videoEl.pause();
            isPlaying.value = false;
        }
    } catch (e) {
        console.error("play/pause failed:", e);
    }
};

/* ---------- filters api ---------- */
const addFilter = (filter) => {
    if (!filter._id)
        filter._id = crypto?.randomUUID
            ? crypto.randomUUID()
            : String(Date.now() + Math.random());
    filters.value = [...filters.value, filter];
};
const removeFilter = (index) => {
    const next = filters.value.slice();
    next.splice(index, 1);
    filters.value = next;
};
const updateFilter = (index, key, value) => {
    const f = filters.value[index];
    if (!f) return;
    const next = filters.value.slice();
    next[index] = {
        ...f,
        params: { ...f.params, [key]: { ...f.params[key], value } },
    };
    filters.value = next;
};

/* ---------- timeline helpers ---------- */
const findTrack = (id) => tracks.value.find((t) => t.id === id);
const newId = () =>
    crypto?.randomUUID
        ? crypto.randomUUID()
        : String(Date.now() + Math.random());

const onAddTrack = (type) => {
    const n = tracks.value.filter((t) => t.type === type).length + 1;
    tracks.value.push({
        id: `t-${type}-${Date.now()}`,
        type,
        label: `${type[0].toUpperCase() + type.slice(1)} ${n}`,
        items: [],
    });
};

const onAddClip = (trackId) => {
    pendingClipTrackId = trackId;
    clipPicker.value?.click();
};
const onPickClip = (e) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;
    const url = URL.createObjectURL(file);
    const isVideo = file.type.startsWith("video/");
    const isImage = file.type.startsWith("image/");
    const tr =
        findTrack(pendingClipTrackId) ||
        tracks.value.find((t) =>
            isVideo ? t.type === "video" : t.type === "image",
        ) ||
        tracks.value[0];
    const start = currentTime.value;
    const end = Math.min(start + 5, duration.value || start + 5);
    tr.items.push({
        id: newId(),
        kind: isVideo ? "video" : "image",
        label: isVideo ? "Video Clip" : "Image",
        src: url,
        start,
        end,
        x: 0.1,
        y: 0.1,
        w: 0.4,
        h: 0.4,
        z: 0,
    });
};

const onAddText = ({ trackId, start, end, text }) => {
    let t = trackId
        ? findTrack(trackId)
        : tracks.value.find((t) => t.type === "text");
    if (!t) {
        t = {
            id: `t-text-${Date.now()}`,
            type: "text",
            label: "Text",
            items: [],
        };
        tracks.value.push(t);
    }
    t.items.push({
        id: newId(),
        kind: "text",
        label: "Text",
        text: text || "New Text",
        color: "#ffffff",
        fontSize: 32,
        fontFamily: "Inter, ui-sans-serif, system-ui",
        x: 0.2,
        y: 0.2,
        w: 0.4,
        h: 0.15,
        z: 10,
        start: start ?? currentTime.value,
        end:
            end ??
            Math.min(
                (currentTime.value || 0) + 3,
                duration.value || currentTime.value + 3,
            ),
    });
};

const onRemoveItem = ({ trackId, itemId }) => {
    const t = findTrack(trackId);
    if (!t) return;
    t.items = t.items.filter((i) => i.id !== itemId);
    selection.value = selection.value.filter(
        (s) => !(s.trackId === trackId && s.itemId === itemId),
    );
};

const onSplit = ({ trackId, itemId, time }) => {
    const t = findTrack(trackId);
    if (!t) return;
    const i = t.items.find((i) => i.id === itemId);
    if (!i) return;
    if (time <= i.start || time >= i.end) return;
    const left = { ...i, id: newId(), end: time };
    const right = { ...i, id: newId(), start: time };
    t.items = t.items.flatMap((it) =>
        it.id === itemId ? [left, right] : [it],
    );
};

const onMerge = ({ trackId, itemIds }) => {
    const t = findTrack(trackId);
    if (!t) return;
    const m = t.items
        .filter((i) => itemIds.includes(i.id))
        .sort((a, b) => a.start - b.start);
    if (m.length < 2) return;
    const merged = {
        ...m[0],
        id: newId(),
        start: m[0].start,
        end: m[m.length - 1].end,
        label: m[0].label || "Merged",
    };
    t.items = [...t.items.filter((i) => !itemIds.includes(i.id)), merged].sort(
        (a, b) => a.start - b.start,
    );
};

const onCut = ({ start, end, selection: sel }) => {
    const byTrack = sel.reduce(
        (m, s) => ((m[s.trackId] ??= []).push(s.itemId), m),
        {},
    );
    for (const [trackId, ids] of Object.entries(byTrack)) {
        const t = findTrack(trackId);
        if (!t) continue;
        const out = [];
        for (const it of t.items) {
            if (!ids.includes(it.id)) {
                out.push(it);
                continue;
            }
            if (it.end <= start || it.start >= end) out.push(it);
            else {
                if (it.start < start)
                    out.push({ ...it, id: newId(), end: start });
                if (it.end > end) out.push({ ...it, id: newId(), start: end });
            }
        }
        t.items = out.sort((a, b) => a.start - b.start);
    }
};

const onSelectionChange = (sel) => {
    selection.value = sel.slice();
};

const onUpdateItem = ({ trackId, itemId, patch }) => {
    const t = findTrack(trackId);
    if (!t) return;
    const idx = t.items.findIndex((i) => i.id === itemId);
    if (idx < 0) return;
    const p = { ...patch };
    if ("x" in p || "y" in p || "w" in p || "h" in p) {
        const base = t.items[idx];
        const x = "x" in p ? p.x : (base.x ?? 0);
        const y = "y" in p ? p.y : (base.y ?? 0);
        const w = "w" in p ? Math.max(0.02, p.w) : (base.w ?? 0.1);
        const h = "h" in p ? Math.max(0.02, p.h) : (base.h ?? 0.1);
        p.x = Math.min(Math.max(x, 0), 1 - w);
        p.y = Math.min(Math.max(y, 0), 1 - h);
        p.w = Math.min(Math.max(w, 0.02), 1);
        p.h = Math.min(Math.max(h, 0.02), 1);
    }
    t.items[idx] = { ...t.items[idx], ...p };
};

const onMoveItem = ({ itemId, fromTrackId, toTrackId, newStart }) => {
    const from = findTrack(fromTrackId);
    const to = findTrack(toTrackId);
    if (!from || !to) return;
    const it = from.items.find((i) => i.id === itemId);
    if (!it) return;
    from.items = from.items.filter((i) => i.id !== itemId);
    to.items.push({ ...it, start: newStart });
};

const onOverlayMove = ({ id, trackId, x, y }) => {
    onUpdateItem({ trackId, itemId: id, patch: { x, y } });
};
const onOverlayResize = ({ id, trackId, w, h }) => {
    onUpdateItem({ trackId, itemId: id, patch: { w, h } });
};
const onOverlaySelect = ({ id, trackId }) => {
    selection.value = [{ trackId, itemId: id }];
};

/* ---------- export ---------- */
const exportVideo = async () => {
    const videoEl = preview.value?.getVideoElement?.();
    if (!videoEl) return;
    exporting.value = true;
    exportProgress.value = 0;
    try {
        if (!videoProcessor && videoEl.videoWidth && videoEl.videoHeight) {
            videoProcessor = new VideoProcessor(
                videoEl.videoWidth,
                videoEl.videoHeight,
            );
        }
        const updateProgress = (p) => {
            const clamped = Math.max(0, Math.min(1, Number(p) || 0));
            exportProgress.value = Math.round(clamped * 100);
        };
        const blob = await export_video(
            videoEl,
            filters.value,
            tracks.value,
            duration.value,
            updateProgress,
        );
        if (!(blob instanceof Blob)) {
            alert(
                "Export not fully implemented yet. Video export will be available in a future update.",
            );
            exporting.value = false;
            return;
        }
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "edited-video.mp4";
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    } catch (e) {
        console.error("Export failed:", e);
        alert("Video export failed. Please try again.");
    } finally {
        exporting.value = false;
    }
};

/* Reset from Inspector */
const onRemoveSelection = ({ trackId, itemId }) =>
    onRemoveItem({ trackId, itemId });
const defaultsFor = (it) => {
    if (it.kind === "text") {
        return {
            text: "New Text",
            color: "#ffffff",
            fontSize: 32,
            fontFamily: "Inter, ui-sans-serif, system-ui",
            x: 0.2,
            y: 0.2,
            w: 0.4,
            h: 0.15,
            z: 10,
        };
    }
    if (it.kind === "image" || it.kind === "video") {
        return { x: 0.1, y: 0.1, w: 0.4, h: 0.4, z: 0 };
    }
    return {};
};
const onResetItem = ({ trackId, itemId }) => {
    const t = findTrack(trackId);
    if (!t) return;
    const idx = t.items.findIndex((i) => i.id === itemId);
    if (idx < 0) return;
    const base = t.items[idx];
    t.items[idx] = { ...base, ...defaultsFor(base) };
};

/* ---------- Keyboard shortcuts ---------- */
const handleKeyDown = (e) => {
    if (["INPUT", "TEXTAREA"].includes(document.activeElement?.tagName)) return;

    switch (e.code) {
        case "Space":
            e.preventDefault();
            onTogglePlay(!isPlaying.value);
            break;
        case "Delete":
            if (selection.value.length > 0) {
                for (const sel of selection.value) {
                    onRemoveItem(sel);
                }
            }
            break;
        case "KeyZ":
            if (e.ctrlKey || e.metaKey) {
                if (e.shiftKey) store.redo();
                else store.undo();
            }
            break;
        case "ArrowLeft":
            seekToTime(Math.max(0, currentTime.value - (e.ctrlKey ? 5 : 0.1)));
            break;
        case "ArrowRight":
            seekToTime(
                Math.min(
                    duration.value,
                    currentTime.value + (e.ctrlKey ? 5 : 0.1),
                ),
            );
            break;
    }
};

onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
});

onBeforeUnmount(() => {
    document.removeEventListener("keydown", handleKeyDown);
});
</script>
