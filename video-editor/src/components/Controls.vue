<!-- src/components/Controls.vue -->
<template>
    <!-- Docked control panel - full height -->
    <aside
        class="h-full rounded-b-xl border border-white/10 bg-white/[0.03] p-4 shadow-[0_10px_40px_-10px_rgba(0,0,0,0.6)] flex flex-col"
    >
        <!-- Title Row - spans all 3 columns -->
        <div class="mb-3 flex-shrink-0">
            <h3 class="text-sm font-semibold text-neutral-200">
                Effects &amp; Corrections
            </h3>
        </div>

        <!-- Main Content - 3 column grid -->
        <div class="flex-1 min-h-0 flex flex-col">
            <div class="grid grid-cols-3 gap-3 h-full">
                <!-- Column 1: Effect Buttons -->
                <div class="flex flex-col gap-2 pr-3 border-r border-white/10">
                    <button
                        v-for="effect in effectButtons"
                        :key="effect.id"
                        @click="addFilterPreset(effect.id)"
                        :disabled="isEffectAdded(effect.id)"
                        :class="[
                            'inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition',
                            isEffectAdded(effect.id)
                                ? 'border-violet-400/50 bg-violet-500/20 text-violet-200 cursor-not-allowed'
                                : 'border-white/10 bg-white/5 text-neutral-200 hover:bg-white/10',
                        ]"
                    >
                        <component
                            :is="effect.icon"
                            :class="[
                                'h-4 w-4',
                                isEffectAdded(effect.id)
                                    ? 'text-violet-300'
                                    : effect.iconColor,
                            ]"
                            aria-hidden="true"
                        />
                        {{ effect.label }}
                    </button>
                </div>

                <!-- Columns 2-3: Filter Control Cards -->
                <div
                    class="col-span-2 flex flex-col gap-3 overflow-y-auto min-h-0 pl-3"
                >
                    <div v-if="filters.length">
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
                                        class="inline-flex items-center justify-center rounded-md border border-white/10 bg-white/5 p-1.5 text-xs text-rose-300 hover:bg-white/10 transition"
                                        title="Remove filter"
                                    >
                                        <TrashIcon
                                            class="h-4 w-4"
                                            aria-hidden="true"
                                        />
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
                        No filters yet. Add one from the left panel.
                    </div>
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
    ArrowDownTrayIcon,
    SparklesIcon,
    BoltIcon,
    AdjustmentsHorizontalIcon,
    XMarkIcon,
    TrashIcon,
    SunIcon,
    EyeIcon,
    CubeIcon,
} from "@heroicons/vue/24/solid";

const props = defineProps({
    filters: { type: Array, default: () => [] },
});

const emit = defineEmits([
    "add-filter",
    "remove-filter",
    "update-filter",
    "export",
]);

// Effect buttons configuration
const effectButtons = [
    {
        id: "grayscale",
        label: "Grayscale",
        icon: SparklesIcon,
        iconColor: "text-violet-400",
    },
    {
        id: "sepia",
        label: "Sepia",
        icon: BoltIcon,
        iconColor: "text-amber-300",
    },
    {
        id: "brightness",
        label: "Brightness",
        icon: AdjustmentsHorizontalIcon,
        iconColor: "text-cyan-300",
    },
    {
        id: "contrast",
        label: "Contrast",
        icon: SunIcon,
        iconColor: "text-orange-400",
    },
    {
        id: "saturation",
        label: "Saturation",
        icon: EyeIcon,
        iconColor: "text-emerald-400",
    },
    { id: "blur", label: "Blur", icon: CubeIcon, iconColor: "text-blue-400" },
];

// Check if an effect is already added
const isEffectAdded = (effectId) => {
    return props.filters.some((f) => f.name === effectId);
};

// ---- helpers
const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const mins = Math.floor(s / 60);
    const secs = Math.floor(s % 60);
    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
};
const toDisplay = (v) =>
    typeof v === "number" && !Number.isInteger(v) ? v.toFixed(2) : v;

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
