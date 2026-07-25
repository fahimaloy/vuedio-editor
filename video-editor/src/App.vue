<!-- src/App.vue -->
<template>
    <div
        id="app"
        class="h-screen flex flex-col bg-neutral-950 text-neutral-100 antialiased"
    >
        <!-- Top App Bar -->
        <header
            class="sticky top-0 z-50 border-b border-white/10 bg-neutral-950/70 backdrop-blur"
        >
            <div class="px-4 sm:px-6 lg:px-8">
                <div class="flex h-14 items-center justify-between">
                    <div class="flex items-center gap-3">
                        <!-- Brand -->
                        <div
                            class="grid h-8 w-8 place-items-center rounded-lg bg-gradient-to-br from-violet-500 to-cyan-400 text-neutral-900 font-black"
                        >
                            V
                        </div>
                        <h1
                            class="text-sm font-semibold tracking-wide text-neutral-200"
                        >
                            Vue WASM <span class="text-neutral-400">|</span>
                            <span class="text-neutral-300">Video Editor</span>
                        </h1>
                    </div>

                    <!-- Right controls -->
                    <div class="flex items-center gap-2">
                        <button
                            class="inline-flex items-center gap-2 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-neutral-200 hover:bg-white/10 transition"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="size-4"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path stroke-width="1.5" d="M12 6v12m6-6H6" />
                            </svg>
                            New Project
                        </button>
                        <button
                            class="inline-flex items-center gap-2 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-neutral-200 hover:bg-white/10 transition"
                        >
                            ⌘S
                            <span class="text-neutral-400">Save</span>
                        </button>
                    </div>
                </div>
            </div>
        </header>

        <!-- Main -->
        <main class="flex-1 overflow-hidden">
            <VideoEditor />
        </main>

        <!-- Status Bar -->
        <footer
            class="sticky bottom-0 z-40 border-t border-white/10 bg-neutral-950/70 backdrop-blur"
        >
            <div class="px-4 sm:px-6 lg:px-8">
                <div class="flex h-10 items-center justify-between text-xs">
                    <p class="text-neutral-400">
                        Powered by <span class="text-neutral-200">Rust</span> •
                        <span class="text-neutral-200">WebAssembly</span>
                    </p>
                    <p class="text-neutral-500">
                        Auto Save <span class="text-emerald-400">●</span>
                    </p>
                </div>
            </div>
        </footer>
    </div>
</template>
<script>
import VideoEditor from "./components/VideoEditor.vue";

export default {
    name: "App",
    components: {
        VideoEditor,
    },
};
</script>
