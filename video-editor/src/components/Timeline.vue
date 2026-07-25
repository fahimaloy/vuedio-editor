<!-- src/components/Timeline.vue -->
<template>
    <section
        class="rounded-xl border border-white/10 bg-white/[0.03] p-3 shadow-[0_10px_40px_-10px_rgba(0,0,0,0.6)] select-none"
    >
        <!-- Header -->
        <div class="flex items-center justify-between gap-3 px-1">
            <div class="flex items-center gap-3">
                <h3 class="text-sm font-semibold text-neutral-200">Timeline</h3>
                <span
                    class="rounded-md border border-white/10 bg-white/5 px-2 py-0.5 text-[11px] font-mono text-neutral-400 whitespace-nowrap"
                >
                    {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
                </span>
            </div>

            <!-- Zoom Controls -->
            <div class="flex items-center gap-2">
                <button
                    @click="zoomOut"
                    class="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-neutral-200 hover:bg-white/10"
                    title="Zoom Out"
                >
                    <MinusIcon class="h-3 w-3" />
                </button>
                <span class="text-[11px] text-neutral-400 font-mono"
                    >{{ Math.round(zoomLevel * 100) }}%</span
                >
                <button
                    @click="zoomIn"
                    class="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-neutral-200 hover:bg-white/10"
                    title="Zoom In"
                >
                    <PlusIcon class="h-3 w-3" />
                </button>

                <!-- Add Clip buttons -->
                <button
                    @click="$emit('add-clip', firstVideoTrackId)"
                    class="rounded-md border border-white/10 bg-emerald-500/90 px-3 py-1.5 text-xs font-medium text-neutral-900 hover:brightness-110 active:scale-95 transition"
                >
                    + Add Clip
                </button>
                <button
                    @click="addTextAtCursor"
                    class="rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-neutral-200 hover:bg-white/10 active:scale-95 transition"
                >
                    + Add Text
                </button>

                <div class="relative">
                    <button
                        @click="trackMenuOpen = !trackMenuOpen"
                        class="rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-neutral-200 hover:bg-white/10 active:scale-95 transition"
                    >
                        + Add Track
                    </button>
                    <div
                        v-if="trackMenuOpen"
                        class="absolute right-0 z-20 mt-2 w-40 overflow-hidden rounded-md border border-white/10 bg-neutral-900/95 p-1 text-sm shadow-xl"
                    >
                        <button
                            class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-neutral-200 hover:bg-white/10"
                            @click="
                                $emit('add-track', 'video');
                                trackMenuOpen = false;
                            "
                        >
                            Video
                        </button>
                        <button
                            class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-neutral-200 hover:bg-white/10"
                            @click="
                                $emit('add-track', 'audio');
                                trackMenuOpen = false;
                            "
                        >
                            Audio
                        </button>
                        <button
                            class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-neutral-200 hover:bg-white/10"
                            @click="
                                $emit('add-track', 'text');
                                trackMenuOpen = false;
                            "
                        >
                            Text
                        </button>
                        <button
                            class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-neutral-200 hover:bg-white/10"
                            @click="
                                $emit('add-track', 'image');
                                trackMenuOpen = false;
                            "
                        >
                            Image
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Ruler (clickable for seeking) -->
        <div
            class="mt-3 overflow-hidden rounded-lg border border-white/10"
            ref="timelineContainer"
        >
            <div class="grid grid-cols-[180px_1fr] bg-neutral-900/60 relative">
                <div class="h-8 border-r border-white/10"></div>
                <div class="relative h-8 cursor-pointer" @click="seekAtEvent">
                    <div
                        class="flex justify-between px-2 text-[11px] text-neutral-400"
                    >
                        <div
                            v-for="t in timeMarkers"
                            :key="t"
                            class="relative flex-1"
                        >
                            <span
                                class="absolute left-0 top-1/2 -translate-y-1/2"
                                >{{ formatTime(t) }}</span
                            >
                            <div
                                class="absolute bottom-0 left-0 h-2 w-px bg-white/10"
                            ></div>
                        </div>
                    </div>
                    <!-- Draggable Playhead -->
                    <div
                        class="absolute inset-y-0 w-0.5 bg-gradient-to-b from-rose-500 to-rose-300 shadow-[0_0_20px_rgba(244,63,94,0.6)] cursor-col-resize"
                        :style="{ left: playheadLeft }"
                        @mousedown="startPlayheadDrag"
                    >
                        <div
                            class="absolute -top-2 -bottom-2 -left-1 -right-1 cursor-col-resize"
                        ></div>
                    </div>
                </div>
            </div>

            <!-- Tracks (vertical scroll) -->
            <div
                class="relative max-h-[46vh] overflow-auto bg-neutral-950/40"
                ref="tracksContainer"
            >
                <div :style="{ width: timelineWidth }">
                    <div class="grid grid-cols-[180px_1fr]">
                        <!-- One row per track -->
                        <template v-for="track in tracks" :key="track.id">
                            <!-- Track header -->
                            <div
                                class="flex items-center gap-2 border-r border-white/5 bg-neutral-900/50 px-3 py-2"
                            >
                                <span
                                    class="truncate text-xs text-neutral-200"
                                    >{{ track.label }}</span
                                >
                                <span
                                    class="ml-auto rounded border border-white/10 bg-white/5 px-2 py-0.5 text-[10px] text-neutral-400"
                                    >{{ track.type }}</span
                                >
                            </div>

                            <!-- Track lane -->
                            <div
                                class="relative h-20 bg-neutral-950/30"
                                @click="seekAtEvent($event)"
                                @dragover.prevent="
                                    onDragOverTrack(track.id, $event)
                                "
                                @drop.prevent="onDropOnTrack(track.id, $event)"
                            >
                                <!-- Playhead line -->
                                <div
                                    class="absolute inset-y-0 w-0.5 bg-gradient-to-b from-rose-500 to-rose-300/80 pointer-events-none"
                                    :style="{ left: playheadLeft }"
                                ></div>

                                <!-- Items -->
                                <div
                                    v-for="item in track.items"
                                    :key="item.id"
                                    class="absolute top-2 h-16 overflow-hidden rounded-md border p-2 text-[11px] shadow backdrop-blur border-white/10"
                                    :class="
                                        isSelected(track.id, item.id)
                                            ? 'bg-white/20 ring-2 ring-violet-400/60 text-neutral-50'
                                            : 'bg-white/10 hover:bg-white/15 text-neutral-100'
                                    "
                                    :style="{
                                        left: toLeft(item.start),
                                        width: toWidth(item.start, item.end),
                                    }"
                                    draggable="true"
                                    @dragstart="
                                        onDragStart(track.id, item, $event)
                                    "
                                    @click.stop="
                                        toggleSelect(track.id, item.id, $event)
                                    "
                                >
                                    <div class="flex items-center gap-2">
                                        <span
                                            class="inline-block h-2 w-2 rounded-full"
                                            :class="
                                                track.type === 'video'
                                                    ? 'bg-violet-300'
                                                    : track.type === 'audio'
                                                      ? 'bg-emerald-300'
                                                      : track.type === 'text'
                                                        ? 'bg-cyan-300'
                                                        : 'bg-amber-300'
                                            "
                                        ></span>
                                        <span class="truncate">{{
                                            item.label ||
                                            defaultItemLabel(track.type)
                                        }}</span>
                                        <button
                                            class="ml-auto rounded border border-white/10 bg-white/10 px-1.5 py-0.5 text-[10px] text-rose-200 hover:bg-white/15"
                                            @click.stop="
                                                $emit('remove-item', {
                                                    trackId: track.id,
                                                    itemId: item.id,
                                                })
                                            "
                                        >
                                            Remove
                                        </button>
                                    </div>
                                    <div
                                        class="mt-1 text-[10px] text-neutral-300/80"
                                    >
                                        {{ formatTime(item.start) }}–{{
                                            formatTime(item.end)
                                        }}
                                    </div>

                                    <!-- resize handles -->
                                    <div
                                        class="absolute inset-y-0 left-0 w-2 cursor-col-resize"
                                        @mousedown.prevent="
                                            startResize(
                                                track.id,
                                                item.id,
                                                'left',
                                                $event,
                                            )
                                        "
                                    ></div>
                                    <div
                                        class="absolute inset-y-0 right-0 w-2 cursor-col-resize"
                                        @mousedown.prevent="
                                            startResize(
                                                track.id,
                                                item.id,
                                                'right',
                                                $event,
                                            )
                                        "
                                    ></div>
                                </div>
                            </div>
                        </template>
                    </div>
                </div>
            </div>
        </div>

        <!-- Ops row -->
        <div class="mt-2 flex items-center gap-2">
            <button
                :disabled="!canSplit"
                @click="
                    $emit('split', {
                        trackId: selectedLocal[0]?.trackId,
                        itemId: selectedLocal[0]?.itemId,
                        time: currentTime,
                    })
                "
                class="rounded-md border border-white/10 px-3 py-1.5 text-xs"
                :class="
                    canSplit
                        ? 'bg-white/5 text-neutral-200 hover:bg-white/10'
                        : 'bg-white/[0.03] text-neutral-500 opacity-60 cursor-not-allowed'
                "
            >
                Split
            </button>
            <button
                :disabled="!canMerge"
                @click="
                    $emit('merge', {
                        trackId: selectedLocal[0]?.trackId,
                        itemIds: selectedLocal.map((s) => s.itemId),
                    })
                "
                class="rounded-md border border-white/10 px-3 py-1.5 text-xs"
                :class="
                    canMerge
                        ? 'bg-white/5 text-neutral-200 hover:bg-white/10'
                        : 'bg-white/[0.03] text-neutral-500 opacity-60 cursor-not-allowed'
                "
            >
                Merge
            </button>
            <button
                :disabled="!canCut"
                @click="emitCutSelection"
                class="rounded-md border border-white/10 px-3 py-1.5 text-xs"
                :class="
                    canCut
                        ? 'bg-white/5 text-neutral-200 hover:bg-white/10'
                        : 'bg-white/[0.03] text-neutral-500 opacity-60 cursor-not-allowed'
                "
            >
                Cut
            </button>
        </div>
    </section>
</template>

<script setup>
import { ref, computed, watch } from "vue";
import { MinusIcon, PlusIcon } from "@heroicons/vue/24/solid";

const props = defineProps({
    duration: { type: Number, default: 0 },
    currentTime: { type: Number, default: 0 },
    tracks: { type: Array, default: () => [] },
    selected: { type: Array, default: () => [] },
});
const emit = defineEmits([
    "seek",
    "add-clip",
    "add-text",
    "add-track",
    "remove-item",
    "split",
    "merge",
    "cut",
    "selection-change",
    "update-item",
    "move-item",
]);

const trackMenuOpen = ref(false);
const zoomLevel = ref(1); // 1 = 100%, 0.5 = 50%, 2 = 200%
const timelineContainer = ref(null);
const tracksContainer = ref(null);

// mirror prop -> local
const selectedLocal = ref([]);
watch(
    () => props.selected,
    (v) => {
        selectedLocal.value = (v || []).slice();
    },
    { immediate: true },
);

const timeMarkers = computed(() => {
    const d = props.duration || 0;
    if (!d) return [];
    const marks = [],
        step = d > 120 ? 10 : d > 30 ? 5 : 1;
    for (let t = 0; t <= d; t += step) marks.push(t);
    return marks;
});

const playheadLeft = computed(() => {
    const d = props.duration || 0,
        t = props.currentTime || 0;
    const pct = d ? (t / d) * 100 : 0;
    return `${Math.min(Math.max(pct, 0), 100)}%`;
});

const timelineWidth = computed(() => {
    // Base width scales with zoom and duration
    const baseWidth = 800; // minimum width
    const zoomedWidth = Math.max(
        baseWidth,
        baseWidth * zoomLevel.value * (props.duration || 1),
    );
    return `${zoomedWidth}px`;
});

const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const m = Math.floor(s / 60),
        sec = Math.floor(s % 60);
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
};
const toLeft = (start) => {
    const d = props.duration || 0;
    const pct = d ? (start / d) * 100 : 0;
    return `${Math.min(Math.max(pct, 0), 100)}%`;
};
const toWidth = (start, end) => {
    const d = props.duration || 0;
    const dur = Math.max(0, end - start);
    const pct = d ? (dur / d) * 100 : 0;
    return `${Math.min(Math.max(pct, 0), 100)}%`;
};

const defaultItemLabel = (type) =>
    type === "video"
        ? "Video"
        : type === "audio"
          ? "Audio"
          : type === "text"
            ? "Text"
            : "Image";
const firstVideoTrackId = computed(
    () => (props.tracks || []).find((t) => t.type === "video")?.id,
);

const isSelected = (trackId, itemId) =>
    selectedLocal.value.some(
        (s) => s.trackId === trackId && s.itemId === itemId,
    );
const toggleSelect = (trackId, itemId, e) => {
    const multi = e?.metaKey || e?.ctrlKey || e?.shiftKey;
    const key = { trackId, itemId };
    if (multi) {
        if (isSelected(trackId, itemId))
            selectedLocal.value = selectedLocal.value.filter(
                (s) => !(s.trackId === trackId && s.itemId === itemId),
            );
        else selectedLocal.value = [...selectedLocal.value, key];
    } else {
        if (isSelected(trackId, itemId) && selectedLocal.value.length === 1)
            selectedLocal.value = [];
        else selectedLocal.value = [key];
    }
    emit("selection-change", selectedLocal.value.slice());
};

const canSplit = computed(() => {
    if (selectedLocal.value.length !== 1) return false;
    const s = selectedLocal.value[0];
    const tr = props.tracks.find((t) => t.id === s.trackId);
    const it = tr?.items.find((i) => i.id === s.itemId);
    return !!(it && props.currentTime > it.start && props.currentTime < it.end);
});
const canMerge = computed(
    () =>
        selectedLocal.value.length >= 2 &&
        selectedLocal.value.every(
            (s) => s.trackId === selectedLocal.value[0].trackId,
        ),
);
const canCut = computed(() => selectedLocal.value.length > 0);
const emitCutSelection = () => {
    const times = [];
    for (const sel of selectedLocal.value) {
        const tr = props.tracks.find((t) => t.id === sel.trackId);
        const it = tr?.items.find((i) => i.id === sel.itemId);
        if (it) {
            times.push(it.start, it.end);
        }
    }
    if (!times.length) return;
    emit("cut", {
        start: Math.min(...times),
        end: Math.max(...times),
        selection: selectedLocal.value.slice(),
    });
};

// Zoom controls
const zoomIn = () => {
    zoomLevel.value = Math.min(10, zoomLevel.value + 0.25);
};
const zoomOut = () => {
    zoomLevel.value = Math.max(0.25, zoomLevel.value - 0.25);
};

// Add text at current time
const addTextAtCursor = () => {
    const t = props.currentTime || 0;
    const d = props.duration || 0;
    const start = t;
    const end = Math.min(t + 3, d || t + 3);
    emit("add-text", { start, end });
};

// Click-to-seek
const seekAtEvent = (evt) => {
    const lane = evt.currentTarget;
    const rect = lane.getBoundingClientRect();
    const x = evt.clientX - rect.left + (lane.scrollLeft || 0);
    const pct = Math.min(Math.max(x / rect.width, 0), 1);
    emit("seek", pct * (props.duration || 0));
};

// Draggable playhead
let playheadDrag = null;
const startPlayheadDrag = (e) => {
    if (e.button !== 0) return;
    const container = timelineContainer.value;
    const rect = container?.getBoundingClientRect();
    if (!rect) return;

    playheadDrag = {
        startX: e.clientX,
        rect,
        scrollLeft: container.scrollLeft,
    };
    window.addEventListener("mousemove", onPlayheadMove);
    window.addEventListener("mouseup", onPlayheadEnd);
};
const onPlayheadMove = (e) => {
    if (!playheadDrag) return;
    const { rect } = playheadDrag;
    const x = e.clientX - rect.left + playheadDrag.scrollLeft;
    const pct = Math.min(Math.max(x / rect.width, 0), 1);
    emit("seek", pct * (props.duration || 0));
};
const onPlayheadEnd = () => {
    playheadDrag = null;
    window.removeEventListener("mousemove", onPlayheadMove);
    window.removeEventListener("mouseup", onPlayheadEnd);
};

/* Dragging across tracks */
let dragPayload = null;
const onDragStart = (fromTrackId, item, e) => {
    dragPayload = { fromTrackId, itemId: item.id };
    try {
        e.dataTransfer.setData("text/plain", "drag");
    } catch {}
};
const onDragOverTrack = (_toTrackId, e) => {
    e.dataTransfer.dropEffect = "move";
};
const onDropOnTrack = (toTrackId, e) => {
    if (!dragPayload) return;
    const lane = e.currentTarget;
    const rect = lane.getBoundingClientRect();
    const x = e.clientX - rect.left + (lane.scrollLeft || 0);
    const pct = Math.min(Math.max(x / rect.width, 0), 1);
    const newStart = pct * (props.duration || 0);
    emit("move-item", {
        itemId: dragPayload.itemId,
        fromTrackId: dragPayload.fromTrackId,
        toTrackId,
        newStart,
    });
    dragPayload = null;
};

/* Resize (left/right) */
let resizeState = null;
const startResize = (trackId, itemId, edge, e) => {
    const lane = e.currentTarget.parentElement?.parentElement;
    const rect = lane?.getBoundingClientRect?.();
    if (!rect) return;
    resizeState = {
        trackId,
        itemId,
        edge,
        startX: e.clientX,
        laneRect: rect,
    };
    window.addEventListener("mousemove", onResizeMove);
    window.addEventListener("mouseup", onResizeEnd);
};
const onResizeMove = (e) => {
    if (!resizeState) return;
    const { trackId, itemId, edge, startX, laneRect } = resizeState;
    const dx = e.clientX - startX;
    const d = props.duration || 0;
    const pxToTime = (dx / laneRect.width) * d;
    let patch = {};
    const tr = props.tracks.find((t) => t.id === trackId);
    const it = tr?.items.find((i) => i.id === itemId);
    if (!it) return;
    if (edge === "left")
        patch = {
            start: Math.min(Math.max(it.start + pxToTime, 0), it.end - 0.1),
        };
    else if (edge === "right")
        patch = {
            end: Math.max(Math.min(it.end + pxToTime, d), it.start + 0.1),
        };
    emit("update-item", { trackId, itemId, patch });
};
const onResizeEnd = () => {
    resizeState = null;
    window.removeEventListener("mousemove", onResizeMove);
    window.removeEventListener("mouseup", onResizeEnd);
};
</script>
