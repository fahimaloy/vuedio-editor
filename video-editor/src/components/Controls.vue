<!-- src/components/Controls.vue -->
<template>
    <!-- Docked control panel - full height -->
    <aside
        class="h-full rounded-xl border border-white/10 bg-white/[0.03] p-4 shadow-[0_10px_40px_-10px_rgba(0,0,0,0.6)] flex flex-col"
    >
        <!-- Row 1: Transport + Time -->
        <div class="flex items-center gap-3 flex-shrink-0">
            <div class="flex items-center gap-2 shrink-0">
                <!-- Play / Pause reflects parent state -->
                <button
                    @click="emit('toggle-play', !playing)"
                    class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/10 text-neutral-100 hover:bg-white/15 active:scale-95 transition"
                    :title="playing ? 'Pause' : 'Play'"
                >
                    <PauseIcon
                        v-if="playing"
                        class="h-5 w-5"
                        aria-hidden="true"
                    />
                    <PlayIcon v-else class="h-5 w-5" aria-hidden="true" />
                </button>

                <!-- -5s -->
                <button
                    @click="emit('seek', Math.max(0, (currentTime || 0) - 5))"
                    class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 active:scale-95 transition"
                    title="Jump -5s"
                >
                    <BackwardIcon class="h-5 w-5" aria-hidden="true" />
                </button>

                <!-- +5s -->
                <button
                    @click="
                        emit(
                            'seek',
                            Math.min(duration || 0, (currentTime || 0) + 5),
                        )
                    "
                    class="inline-flex size-9 items-center justify-center rounded-md border border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10 active:scale-95 transition"
                    title="Jump +5s"
                >
                    <ForwardIcon class="h-5 w-5" aria-hidden="true" />
                </button>
            </div>

            <!-- Time readout (single line) -->
            <div
                class="ml-2 inline-flex items-center gap-2 rounded-md border border-white/10 bg-white/5 px-3 py-1 text-[12px] font-mono text-neutral-300 whitespace-nowrap"
            >
                <span>{{ formatTime(currentTime) }}</span>
                <span class="text-neutral-500">/</span>
                <span class="text-neutral-200">{{ formatTime(duration) }}</span>
            </div>
        </div>

        <!-- Row 2: Scrubber (separate row as requested) -->
        <div class="mt-3 pt-3 border-t border-white/10 flex-shrink-0">
            <div class="flex items-center gap-3 min-w-0">
                <input
                    type="range"
                    min="0"
                    :max="duration || 0"
                    :value="currentTime || 0"
                    @input="handleSeek"
                    class="min-w-0 w-0 flex-1 appearance-none bg-white/10 h-2 rounded-full outline-none cursor-pointer"
                    aria-label="Timeline scrubber"
                />
                <input
                    type="number"
                    :value="Math.round(currentTime || 0)"
                    @input="emit('seek', Number($event.target.value) || 0)"
                    class="w-16 shrink-0 rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-right text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                    :min="0"
                    :max="duration || 0"
                    aria-label="Current time in seconds"
                />
                <span
                    class="text-xs text-neutral-400 font-mono w-12 text-right"
                    aria-live="polite"
                >
                    {{ formatTime(currentTime || 0) }}
                </span>
            </div>
        </div>

        <!-- Filters - scrollable content -->
        <div class="pt-4 flex-1 min-h-0 flex flex-col">
            <div
                class="mb-3 flex items-center justify-between gap-2 flex-shrink-0"
            >
                <h3 class="text-sm font-semibold text-neutral-200">
                    Effects &amp; Corrections
                </h3>
                <div class="flex flex-wrap items-center gap-2">
                    <button
                        @click="addFilterPreset('grayscale')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        <SparklesIcon
                            class="h-4 w-4 text-violet-400"
                            aria-hidden="true"
                        />
                        Grayscale
                    </button>
                    <button
                        @click="addFilterPreset('sepia')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        <BoltIcon
                            class="h-4 w-4 text-amber-300"
                            aria-hidden="true"
                        />
                        Sepia
                    </button>
                    <button
                        @click="addFilterPreset('brightness')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        <AdjustmentsHorizontalIcon
                            class="h-4 w-4 text-cyan-300"
                            aria-hidden="true"
                        />
                        Brightness
                    </button>
                    <button
                        @click="addFilterPreset('contrast')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        Contrast
                    </button>
                    <button
                        @click="addFilterPreset('saturation')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        Saturation
                    </button>
                    <button
                        @click="addFilterPreset('blur')"
                        class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-neutral-200 hover:bg-white/10 transition"
                    >
                        Blur
                    </button>
                </div>
            </div>

            <div class="flex-1 min-h-0 overflow-y-auto">
                <div v-if="filters.length" class="grid gap-3">
                    <div
                        v-for="(filter, index) in filters"
                        :key="filter._id ?? `${filter.name}-${index}`"
                        class="overflow-hidden rounded-lg border border-white/10 bg-neutral-900/60 p-3"
                    >
                        <div class="mb-3 flex items-center justify-between">
                            <div class="flex items-center gap-2 min-w-0">
                                <span
                                    class="inline-flex size-2.5 rounded-full bg-violet-400/80"
                                ></span>
                                <span
                                    class="text-sm font-medium text-neutral-200 truncate"
                                    >{{ filter.label }}</span
                                >
                                <span
                                    class="text-[11px] text-neutral-400 hidden sm:inline"
                                    >({{ filter.name }})</span
                                >
                            </div>
                            <div class="flex items-center gap-2">
                                <button
                                    @click="emit('remove-filter', index)"
                                    class="inline-flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-rose-300 hover:bg-white/10 transition"
                                    title="Remove filter"
                                >
                                    <XMarkIcon
                                        class="h-4 w-4"
                                        aria-hidden="true"
                                    />
                                    Remove
                                </button>
                            </div>
                        </div>

                        <!-- Parameter rows -->
                        <div v-if="filter.params" class="grid gap-3">
                            <div
                                v-for="(param, key) in filter.params"
                                :key="key"
                                class="grid grid-cols-12 items-center gap-3"
                            >
                                <!-- label -->
                                <label
                                    class="col-span-12 sm:col-span-3 text-[12px] text-neutral-400"
                                >
                                    {{ key }}
                                </label>

                                <!-- slider + value pill -->
                                <div
                                    class="col-span-9 sm:col-span-7 flex items-center gap-3 min-w-0"
                                >
                                    <input
                                        type="range"
                                        v-model.number="param.value"
                                        :min="param.min"
                                        :max="param.max"
                                        :step="param.step || 1"
                                        @input="
                                            emit(
                                                'update-filter',
                                                index,
                                                key,
                                                param.value,
                                            )
                                        "
                                        class="min-w-0 w-0 flex-1 appearance-none bg-white/10 h-2 rounded-full outline-none cursor-pointer"
                                        :aria-label="`Filter ${filter.label} ${key}`"
                                    />
                                    <span
                                        class="min-w-12 text-right text-[11px] font-mono text-neutral-300"
                                    >
                                        {{ toDisplay(param.value) }}
                                    </span>
                                </div>

                                <!-- numeric box -->
                                <div class="col-span-3 sm:col-span-2">
                                    <input
                                        type="number"
                                        v-model.number="
                                            filter.params[key].value
                                        "
                                        :min="param.min"
                                        :max="param.max"
                                        :step="param.step || 1"
                                        @input="
                                            emit(
                                                'update-filter',
                                                index,
                                                key,
                                                filter.params[key].value,
                                            )
                                        "
                                        class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-right text-xs text-neutral-100 outline-none focus:ring-2 focus:ring-violet-500/40"
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div
                    v-else
                    class="h-full rounded-lg border border-dashed border-white/10 bg-white/[0.02] p-6 text-center text-xs text-neutral-500 flex items-center justify-center"
                >
                    No filters yet. Add one above to get started.
                </div>
            </div>
        </div>

        <!-- Export -->
        <div class="mt-4 flex items-center justify-end gap-2 flex-shrink-0">
            <button
                @click="emit('export')"
                class="inline-flex items-center gap-2 rounded-md border border-white/10 bg-gradient-to-r from-violet-600 to-cyan-400 px-4 py-2 text-sm text-neutral-950 font-semibold shadow hover:brightness-105 active:scale-95 transition"
            >
                <ArrowDownTrayIcon class="h-4 w-4" aria-hidden="true" />
                Export Video
            </button>
        </div>
    </aside>
</template>

<script setup>
import {
    PlayIcon,
    PauseIcon,
    BackwardIcon,
    ForwardIcon,
    ArrowDownTrayIcon,
    SparklesIcon,
    BoltIcon,
    AdjustmentsHorizontalIcon,
    XMarkIcon,
} from "@heroicons/vue/24/solid";

const props = defineProps({
    playing: { type: Boolean, default: false }, // parent-owned play state
    filters: { type: Array, default: () => [] },
    duration: { type: Number, default: 0 },
    currentTime: { type: Number, default: 0 },
});

const emit = defineEmits([
    "add-filter",
    "remove-filter",
    "update-filter",
    "seek",
    "export",
    "toggle-play",
]);

// ---- helpers
const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const mins = Math.floor(s / 60);
    const secs = Math.floor(s % 60);
    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
};
const toDisplay = (v) =>
    typeof v === "number" && !Number.isInteger(v) ? v.toFixed(2) : v;
const handleSeek = (e) => emit("seek", parseFloat(e.target.value || "0"));

// Presets → ensure a stable _id so remove works reliably
const addFilterPreset = (which) => {
    let preset;
    if (which === "grayscale") {
        preset = {
            name: "grayscale",
            label: "Grayscale",
            params: { intensity: { value: 100, min: 0, max: 100, step: 1 } },
        };
    } else if (which === "sepia") {
        preset = {
            name: "sepia",
            label: "Sepia",
            params: { intensity: { value: 80, min: 0, max: 100, step: 1 } },
        };
    } else if (which === "brightness") {
        preset = {
            name: "brightness",
            label: "Brightness",
            params: { level: { value: 1.0, min: 0.1, max: 3.0, step: 0.1 } },
        };
    } else if (which === "contrast") {
        preset = {
            name: "contrast",
            label: "Contrast",
            params: { level: { value: 1.0, min: 0.0, max: 3.0, step: 0.1 } },
        };
    } else if (which === "saturation") {
        preset = {
            name: "saturation",
            label: "Saturation",
            params: { level: { value: 1.0, min: 0.0, max: 3.0, step: 0.1 } },
        };
    } else if (which === "blur") {
        preset = {
            name: "blur",
            label: "Blur",
            params: { radius: { value: 2, min: 0, max: 20, step: 1 } },
        };
    } else {
        return;
    }
    const f = JSON.parse(JSON.stringify(preset));
    f._id =
        globalThis.crypto && crypto.randomUUID
            ? crypto.randomUUID()
            : String(Date.now() + Math.random());
    emit("add-filter", f);
};
</script>
