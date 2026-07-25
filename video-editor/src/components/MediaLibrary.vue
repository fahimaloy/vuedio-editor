<!-- src/components/MediaLibrary.vue -->
<template>
    <aside class="rounded-xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-3 flex items-center justify-between">
            <h3 class="text-sm font-semibold text-neutral-200">
                Media Library
            </h3>
            <button
                @click="showUploader = !showUploader"
                class="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-neutral-200 hover:bg-white/10"
            >
                {{ showUploader ? "Close" : "+ Add" }}
            </button>
        </div>

        <!-- Uploader -->
        <div v-if="showUploader" class="mb-3">
            <input
                ref="fileInput"
                type="file"
                accept="video/*,image/*,audio/*"
                multiple
                @change="handleFiles"
                class="hidden"
            />
            <label
                for="media-upload"
                class="flex cursor-pointer flex-col items-center gap-2 rounded-lg border border-dashed border-white/10 bg-white/[0.02] p-4 text-center"
            >
                <input
                    id="media-upload"
                    type="file"
                    accept="video/*,image/*,audio/*"
                    multiple
                    @change="handleFiles"
                    class="hidden"
                />
                <div class="text-xs text-neutral-400">
                    Drop files or click to upload
                </div>
                <div class="text-[10px] text-neutral-500">
                    MP4, WebM, MOV, PNG, JPG, WAV, MP3
                </div>
            </label>
        </div>

        <!-- Media list -->
        <div v-if="media.length === 0" class="text-xs text-neutral-500">
            No media assets. Add some above.
        </div>
        <div v-else class="space-y-2 max-h-60 overflow-y-auto">
            <div
                v-for="item in media"
                :key="item.id"
                class="flex items-center gap-2 rounded-md border border-white/10 bg-neutral-900/50 p-2 cursor-grab active:cursor-grabbing"
                draggable="true"
                @dragstart="onDragStart(item, $event)"
            >
                <div
                    class="flex-shrink-0 size-10 rounded bg-neutral-800 grid place-items-center"
                >
                    <VideoCameraIcon
                        v-if="item.type === 'video'"
                        class="h-5 w-5 text-violet-400"
                    />
                    <MusicalNoteIcon
                        v-else-if="item.type === 'audio'"
                        class="h-5 w-5 text-emerald-400"
                    />
                    <PhotoIcon v-else class="h-5 w-5 text-amber-400" />
                </div>
                <div class="min-w-0 flex-1">
                    <div class="truncate text-xs text-neutral-200">
                        {{ item.name }}
                    </div>
                    <div class="text-[10px] text-neutral-400">
                        {{ item.type }}
                    </div>
                </div>
                <button
                    @click="$emit('remove', item.id)"
                    class="rounded p-1 text-[10px] text-rose-300 hover:bg-white/10"
                >
                    <XMarkIcon class="h-3 w-3" />
                </button>
            </div>
        </div>
    </aside>
</template>

<script setup>
import { ref, computed } from "vue";
import {
    VideoCameraIcon,
    MusicalNoteIcon,
    PhotoIcon,
    XMarkIcon,
} from "@heroicons/vue/24/outline";
import { useEditorStore } from "../stores/editor";

const store = useEditorStore();
const showUploader = ref(false);
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
</script>
