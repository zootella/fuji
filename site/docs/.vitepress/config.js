import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
	title: 'Fuji',
	description: 'A multimedia file manager designed with privacy and precision in mind',

	// Trailing-slash-free URLs; the server resolves /page to page.html
	cleanUrls: true,

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
