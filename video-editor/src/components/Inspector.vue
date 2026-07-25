<!-- src/components/Inspector.vue -->
<template>
    <aside
        class="h-full rounded-b-xl border border-white/10 bg-white/[0.03] p-4 flex flex-col"
    >
        <div class="mb-3 flex items-center justify-between flex-shrink-0">
            <h3 class="text-sm font-semibold text-neutral-200">Inspector</h3>
            <button
                v-if="current"
                @click="resetToDefaults"
                class="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-neutral-200 hover:bg-white/10"
            >
                Reset
            </button>
        </div>

        <div v-if="!current" class="flex-1 flex items-center justify-center">
            <div class="text-center">
                <div
                    class="mx-auto mb-3 grid size-12 place-items-center rounded-xl bg-white/5"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6 text-neutral-400"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="1.5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M9 10l4.553-2.276A1 1 0 0115 8.618v6.764a1 1 0 01-1.447.894L9 14"
                        />
                    </svg>
                </div>
                <p class="text-xs text-neutral-500">
                    Select an item to edit its properties.
                </p>
            </div>
        </div>

        <div v-else class="flex-1 min-h-0 overflow-y-auto space-y-3">
            <div class="grid grid-cols-2 gap-2">
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Label</label
                    >
                    <input
                        v-model="draft.label"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                    />
                </div>
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Z Index</label
                    >
                    <input
                        type="number"
                        v-model.number="draft.z"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                    />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-2">
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Start (s)</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        v-model.number="draft.start"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                    />
                </div>
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >End (s)</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        v-model.number="draft.end"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                    />
                </div>
            </div>

            <div class="grid grid-cols-4 gap-2">
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >X Position</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        min="0"
                        max="1"
                        v-model.number="draft.x"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                    />
                </div>
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Y Position</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        min="0"
                        max="1"
                        v-model.number="draft.y"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                    />
                </div>
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Width</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        min="0.02"
                        max="1"
                        v-model.number="draft.w"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                    />
                </div>
                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Height</label
                    >
                    <input
                        type="number"
                        step="0.01"
                        min="0.02"
                        max="1"
                        v-model.number="draft.h"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                    />
                </div>
            </div>

            <!-- Text-only fields -->
            <div
                v-if="current.kind === 'text'"
                class="space-y-2 pt-2 border-t border-white/10"
            >
                <!-- Text formatting toolbar -->
                <div class="flex items-center gap-1">
                    <button
                        @click="toggleBold"
                        :class="
                            draft.fontWeight >= 700
                                ? 'bg-white/20'
                                : 'bg-white/5'
                        "
                        class="rounded border border-white/10 px-2 py-1 text-xs font-bold hover:bg-white/15"
                        title="Bold"
                    >
                        B
                    </button>
                    <button
                        @click="toggleItalic"
                        :class="
                            draft.fontStyle === 'italic'
                                ? 'bg-white/20'
                                : 'bg-white/5'
                        "
                        class="rounded border border-white/10 px-2 py-1 text-xs italic hover:bg-white/15"
                        title="Italic"
                    >
                        I
                    </button>
                    <select
                        v-model="draft.textAlign"
                        class="rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100"
                    >
                        <option value="left">Left</option>
                        <option value="center">Center</option>
                        <option value="right">Right</option>
                    </select>
                </div>

                <div>
                    <label class="block text-[11px] text-neutral-400 mb-1"
                        >Text</label
                    >
                    <textarea
                        v-model="draft.text"
                        rows="3"
                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                    ></textarea>
                </div>
                <div class="grid grid-cols-3 gap-2">
                    <div>
                        <label class="block text-[11px] text-neutral-400 mb-1"
                            >Color</label
                        >
                        <input
                            type="color"
                            v-model="draft.color"
                            class="h-8 w-full rounded-md border border-white/10 bg-neutral-900"
                        />
                    </div>
                    <div>
                        <label class="block text-[11px] text-neutral-400 mb-1"
                            >Font Size</label
                        >
                        <input
                            type="number"
                            v-model.number="draft.fontSize"
                            class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                        />
                    </div>
                    <div>
                        <label class="block text-[11px] text-neutral-400 mb-1"
                            >Font Family</label
                        >
                        <input
                            v-model="draft.fontFamily"
                            class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"
                        />
                    </div>
                </div>

                <!-- Text styling extras -->
                <div class="grid grid-cols-2 gap-2">
                    <div>
                        <label class="block text-[11px] text-neutral-400 mb-1"
                            >Background</label
                        >
                        <input
                            type="color"
                            v-model="draft.backgroundColor"
                            class="h-8 w-full rounded-md border border-white/10 bg-neutral-900"
                        />
                    </div>
                    <div>
                        <label class="block text-[11px] text-neutral-400 mb-1"
                            >Opacity</label
                        >
                        <input
                            type="range"
                            min="0"
                            max="100"
                            v-model.number="draft.opacity"
                            class="w-full"
                        />
                    </div>
                </div>
            </div>

            <div class="flex items-center justify-end gap-2 pt-2 flex-shrink-0">
                <button
                    @click="emitRemove"
                    class="rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-rose-300 hover:bg-white/10"
                >
                    Remove
                </button>
            </div>
        </div>
    </aside>
</template>

<script setup>
import { computed, reactive, watch } from "vue";

const props = defineProps({
    selected: { type: Array, default: () => [] }, // [{trackId,itemId}]
    tracks: { type: Array, default: () => [] },
    duration: { type: Number, default: 0 },
});
const emit = defineEmits(["update-item", "remove", "reset-item"]);

const current = computed(() => {
    if (!props.selected?.length) return null;
    const { trackId, itemId } = props.selected[0];
    const t = props.tracks.find((t) => t.id === trackId);
    return t?.items.find((i) => i.id === itemId) || null;
});

const draft = reactive({});

// Fill draft when selection changes - use flush: 'post' for proper timing
watch(
    current,
    (c) => {
        // Clear existing keys
        Object.keys(draft).forEach((k) => delete draft[k]);
        if (c) {
            // Copy all properties to draft
            Object.entries(c).forEach(([k, v]) => {
                draft[k] = v;
            });
        }
    },
    { immediate: true, flush: "post" },
);

// Realtime update: whenever draft changes, emit patch with proper change detection
let pendingPatch = null;
const scheduleUpdate = () => {
    if (pendingPatch) cancelAnimationFrame(pendingPatch);
    pendingPatch = requestAnimationFrame(() => {
        if (!current.value) return;
        const { trackId, itemId } = props.selected[0];
        const patch = {};

        // Build patch from changed properties
        for (const [k, v] of Object.entries(draft)) {
            if (v !== current.value[k]) {
                patch[k] = v;
            }
        }

        if (Object.keys(patch).length === 0) return;

        // Clamp geometry here (0..1 and min size) to prevent hiding
        if ("x" in patch || "y" in patch || "w" in patch || "h" in patch) {
            const x = patch.x ?? current.value.x ?? 0;
            const y = patch.y ?? current.value.y ?? 0;
            const w = Math.min(
                Math.max(patch.w ?? current.value.w ?? 0.1, 0.02),
                1,
            );
            const h = Math.min(
                Math.max(patch.h ?? current.value.h ?? 0.1, 0.02),
                1,
            );
            patch.x = Math.min(Math.max(x, 0), 1 - w);
            patch.y = Math.min(Math.max(y, 0), 1 - h);
            patch.w = w;
            patch.h = h;
        }

        emit("update-item", { trackId, itemId, patch });
        pendingPatch = null;
    });
};

watch(draft, scheduleUpdate, { deep: true });

const emitRemove = () => {
    if (!current.value) return;
    emit("remove", {
        trackId: props.selected[0].trackId,
        itemId: props.selected[0].itemId,
    });
};

const resetToDefaults = () => {
    if (!current.value) return;
    emit("reset-item", {
        trackId: props.selected[0].trackId,
        itemId: props.selected[0].itemId,
    });
};

// Text formatting helpers
const toggleBold = () => {
    draft.fontWeight = draft.fontWeight >= 700 ? 400 : 700;
};
const toggleItalic = () => {
    draft.fontStyle = draft.fontStyle === "italic" ? "normal" : "italic";
};
</script>
