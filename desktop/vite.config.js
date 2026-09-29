import {defineConfig} from "vite"
import vue from "@vitejs/plugin-vue"
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [
		vue(),
		tailwindcss(),
	],

	// imports among fuji's own files point one way, so every module has run by the time anything that imports it runs; a loop fails the build rather than shipping. Loops inside node_modules are those libraries' own business
	build: {
		rolldownOptions: {
			checks: {circularDependency: true},
			onLog(level, log, handler) {
				if (log.code == 'CIRCULAR_DEPENDENCY' && !log.message.includes('node_modules')) throw new Error(log.message)
				handler(level, log)
			},
		},
	},

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host ? {protocol: "ws", host, port: 1421} : undefined,
		watch: {
			// 3. tell vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"],
		},
	},
})
