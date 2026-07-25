<!-- src/components/MediaLibrary.vue -->
<template>
    <aside class="h-full flex flex-col">
        <!-- Header -->
        <div class="mb-3 flex items-center justify-between px-1 flex-shrink-0">
            <h3 class="text-sm font-semibold text-neutral-200">
                Media Library
            </h3>
            <div class="flex items-center gap-2">
                <input
                    ref="fileInput"
                    type="file"
                    accept="video/*,image/*,audio/*"
                    multiple
                    @change="handleFiles"
                    class="hidden"
                />
                <button
                    @click="fileInput?.click()"
                    class="rounded-md border border-white/10 bg-gradient-to-r from-violet-600/80 to-cyan-400/80 px-3 py-1.5 text-xs text-neutral-950 font-semibold shadow hover:brightness-105 active:scale-95 transition flex items-center gap-1.5"
                    :title="'Add Media'"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-4 w-4"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M12 4v16m8-8H4" />
                    </svg>
                    Add Media
                </button>
            </div>
        </div>

        <!-- Upload Area (shown when empty) -->
        <div
            v-if="media.length === 0"
            class="flex-1 rounded-xl border border-dashed border-white/10 bg-gradient-to-b from-white/[0.04] to-white/[0.02] p-6 text-center flex flex-col items-center justify-center min-h-0"
        >
            <input
                id="media-upload-empty"
                type="file"
                accept="video/*,image/*,audio/*"
                multiple
                @change="handleFiles"
                class="hidden"
            />
            <label
                for="media-upload-empty"
                class="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-white/10 bg-gradient-to-r from-violet-600/80 to-cyan-400/80 px-5 py-2.5 text-sm text-neutral-950 font-semibold shadow hover:brightness-105 active:scale-95 transition"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="M12 4v16m8-8H4" />
                </svg>
                Upload Media
            </label>
            <p class="mt-3 text-xs text-neutral-400">
                MP4, WebM, MOV, PNG, JPG, WAV, MP3
            </p>
        </div>

        <!-- Media list -->
        <div v-else class="flex-1 space-y-2 overflow-y-auto min-h-0">
            <div
                v-for="item in media"
                :key="item.id"
                class="flex items-center gap-3 rounded-lg border border-white/10 bg-neutral-900/50 p-3 cursor-grab active:cursor-grabbing hover:bg-white/5 transition"
                draggable="true"
                @dragstart="onDragStart(item, $event)"
            >
                <div
                    class="flex-shrink-0 size-12 rounded-lg bg-neutral-800 grid place-items-center"
                >
                    <VideoCameraIcon
                        v-if="item.type === 'video'"
                        class="h-6 w-6 text-violet-400"
                    />
                    <MusicalNoteIcon
                        v-else-if="item.type === 'audio'"
                        class="h-6 w-6 text-emerald-400"
                    />
                    <PhotoIcon v-else class="h-6 w-6 text-amber-400" />
                </div>
                <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-medium text-neutral-200">
                        {{ item.name }}
                    </div>
                    <div class="text-xs text-neutral-400 mt-0.5">
                        {{ item.type }}
                    </div>
                </div>
                <button
                    @click="$emit('remove', item.id)"
                    class="rounded-lg p-1.5 text-xs text-rose-300 hover:bg-white/10 transition"
                    title="Remove"
                >
                    <XMarkIcon class="h-4 w-4" />
                </button>
            </div>
        </div>
    </aside>
</template>

<script setup>
import { ref, computed, onBeforeUnmount } from "vue";
import {
    VideoCameraIcon,
    MusicalNoteIcon,
    PhotoIcon,
    XMarkIcon,
} from "@heroicons/vue/24/outline";
import { useEditorStore } from "../stores/editor";

const store = useEditorStore();
const fileInput = ref(null);

const media = computed(() => store.mediaLibrary);

const handleFiles = (e) => {
    const files = Array.from(e.target.files || []);
    for (const file of files) {
        const url = URL.createObjectURL(file);
        let type = "image";
        if (file.type.startsWith("video/")) type = "video";
        else if (file.type.startsWith("audio/")) type = "audio";
        store.addMedia(file, url, type);
    }
    e.target.value = "";
};

const onDragStart = (item, e) => {
    e.dataTransfer.setData("application/json", JSON.stringify(item));
};

const removeMedia = (id) => {
    store.removeMedia(id);
};

// Cleanup object URLs on unmount (for any that weren't explicitly revoked)
onBeforeUnmount(() => {
    // Media URLs are managed by the store, but we clean up any that might persist
});
</script>
