import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

//the server for the palette, which draws with fuji's own stylesheet, components and faces, and so reaches into desktop for them
export default defineConfig({
	plugins: [
		vue(),
		tailwindcss(),
	],
	publicDir: '../desktop/public',//the faces the stylesheet fetches from /fonts
	resolve: {dedupe: ['vue']},//one copy of Vue for the palette's own files and fuji's components alike, so a component is never drawn by a Vue that does not know it
	server: {
		strictPort: true,
		fs: {allow: ['..']},//the repository, since fuji's files are outside this workspace
	},
})
