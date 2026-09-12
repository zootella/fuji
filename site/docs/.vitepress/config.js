import { defineConfig } from 'vitepress'

// Every extension tauri can bundle, so the client router treats these as files to fetch rather than
// pages to route to. Its own list happens to know exe and zip but not dmg, deb or appimage, and without
// this it resolves /fuji.dmg as a route and asks the server for fuji.dmg.html. Naming all of them,
// including the two it already knows, means this does not depend on what is on somebody else's list.
// Extensions we do not ship are here on purpose: nothing will ever be a page named .rpm, and listing
// one before the link exists is the only way a future download cannot arrive broken.
//
// Two things to know. Lowercase only — the lookup lowercases the extension it finds, but matches the
// list as written, so .AppImage is caught by appimage and never by AppImage. And this is set here
// rather than in an env file so that the site workspace keeps no .env of its own for vite to read.
process.env.VITE_EXTRA_EXTENSIONS = 'dmg,exe,msi,deb,rpm,appimage,sig,zip'

// https://vitepress.dev/reference/site-config
export default defineConfig({
	title: 'Fuji',
	description: 'A multimedia file manager designed with privacy and precision in mind',

	// cleanUrls stays off, which is vitepress's default: every link carries its .html, so every link
	// points at a file that exists and the site needs no rule on the server to resolve a bare path

	head: [
		// the mint disc, the same mark as the application icon
		['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],

		//IBM Plex Mono, for the copy on the home page. Helvetica Neue is packaged with the repository; this one is still fetched, as it was before.
		['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
		['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
		['link', { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,300;0,400;0,700;1,300;1,400;1,700&display=swap' }],
	],

	themeConfig: {
		// https://vitepress.dev/reference/default-theme-config
		nav: [
			{ text: 'Home', link: '/' },
		],

		sidebar: [
			{
				text: 'Examples',
				items: [
					{ text: 'Markdown Examples', link: '/markdown-examples' },
					{ text: 'Runtime API Examples', link: '/api-examples' },
				],
			},
		],

		// the application's repository, the same one the home page's GitHub link points at
		socialLinks: [
			{ icon: 'github', link: 'https://github.com/zootella/fuji' },
		],
	},
})
