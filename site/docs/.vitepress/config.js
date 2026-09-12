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

/*
The syntax colors. Shiki highlights every fence at build time using a real VS Code theme file, so this
is the same operation as changing a theme in an editor, by name, from the fifty-four Shiki bundles. The
colors end up inline on each token as --shiki-light and --shiki-dark custom properties, both palettes on
the page at once, which is why the appearance toggle never flashes — and why changing this needs a
rebuild rather than a stylesheet edit.

Each theme's own background is discarded: VitePress paints the block from --vp-code-block-bg, so a theme
has to read well on our near-white and near-black rather than on the cream or slate its author chose.
That is also why this is a pair and not one name.

Catppuccin was chosen by walking the alternatives. What it has is even chroma: the project is built as
one fixed set of accents held at a constant saturation and lightness across every hue, then assigned to
scopes, so no token shouts over its neighbours. Measured across seven token colors, mean saturation is
82% in latte and 78% in mocha against Solarized's 73%, and mocha's accents average 77% lightness where
fuji's own mint is 81% — the same chroma with far more light behind it, which is what reads as neon.

It also italicizes comments, and that matters more than it looks: the two rules in theme/style.css that
carry a theme's fontStyle through to the page do nothing unless the theme asks for italic. Of the light
themes in the bundle only seven do, and no github-* variant is among them.

What was tried and set aside, in the order it was tried:

	github-light / github-dark             VitePress's default and the site's starting point; no italics
	solarized-light / solarized-dark       the runner-up. The only mode-symmetric pair — identical token
	                                       colors in both modes, only the comment grey moves — and the
	                                       only one putting fuji's hue on tokens you read constantly:
	                                       teal strings 14° from the mint, blue functions 44°. Lost on
	                                       its keywords, a 100%-saturated olive 93° away, everywhere
	snazzy-light / monokai                 the loudest, and the reason the italics went missing
	material-theme-lighter / -ocean        cyan-forward but earthy and pastel
	one-light / one-dark-pro               crayon rather than permanent marker
	everforest-light / everforest-dark     earthy
	rose-pine-dawn / rose-pine             muted; three token scopes land on one colour

Further out, if the brief ever changes: light-plus and dark-plus are Visual Studio's primary palette,
keyword #0000ff, the crudest colors in the bundle and no italics anywhere. And markdown.theme takes a
theme object as readily as a name, so italics can be patched onto any of these — it needs @shikijs/themes
as a dependency of this workspace, since pnpm will not let us reach the copy inside vitepress.
*/
const markdown = {
	theme: {light: 'catppuccin-latte', dark: 'catppuccin-mocha'},
}

// https://vitepress.dev/reference/site-config
export default defineConfig({
	markdown,

	title: 'Fuji',
	description: 'A multimedia file manager designed with privacy and precision in mind',

	// cleanUrls stays off, which is vitepress's default: every link carries its .html, so every link
	// points at a file that exists and the site needs no rule on the server to resolve a bare path

	head: [
		// the mint disc, the same mark as the application icon
		['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],

		// and nothing else. Every face the site sets text in is packaged with the repository and declared
		// in theme/style.css, so there is no stylesheet here to fetch from anybody. IBM Plex Mono used to
		// arrive as a google fonts link with preconnect hints to two of their hosts beside it; the files
		// are local now, which is what let all three lines go.
	],

	themeConfig: {
		// https://vitepress.dev/reference/default-theme-config
		nav: [
			{ text: 'Home', link: '/' },
		],

		// Three sections. User Guide is how to use fuji, Craftsmanship is how one part of it is built
		// and why, with what was measured to decide it, and About is everything else. Two of the
		// pages below are stubs carrying only their titles.
		//
		// Examples is scaffolding rather than a fourth section: VitePress's own two starter pages,
		// and the Code Examples page of real fuji source that the site's type is being judged
		// against. All three go when they have served their turn.
		//
		// Links are written without .html and VitePress appends it, because cleanUrls is off above.
		sidebar: [
			{
				text: 'User Guide',
				items: [
					{ text: 'Getting Started', link: '/getting-started' },
				],
			},
			{
				text: 'Craftsmanship',
				items: [
					{ text: 'The thumbnail pipeline', link: '/thumbnail-pipeline' },
				],
			},
			{
				text: 'About',
				items: [
					{ text: 'Meet Aki', link: '/meet-aki' },
				],
			},
			{
				text: 'Examples',
				items: [
					{ text: 'Code Examples', link: '/code-examples' },
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
