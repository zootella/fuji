<script setup>//./.vitepress/theme/components/LastUpdated.vue
// The "Last updated" line under every article, in place of VitePress's own, which config.js swaps for this file through a Vite alias, the way VitePress documents for its internal components. VitePress formats the date with Intl in the reader's locale, "10/9/26, 1:04 AM" for one reader and another form for the next, and can only fill the words in after hydration, since a locale is known only in the browser. This site writes a date one way everywhere, the way the download page writes a sidecar's: 2026 October 11, the day and no hour, since the hour of a commit tells a reader nothing. The timestamp is the one VitePress reads from git at build time, the page's last commit, and the day is taken in UTC, which is the same on the server and in every browser, so the built page already carries the words and nothing changes on hydration.
import { computed } from 'vue'
import { useData } from 'vitepress'

const { theme, page } = useData()
const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December']
const date = computed(() => new Date(page.value.lastUpdated))
const iso = computed(() => date.value.toISOString())
const said = computed(() => `${date.value.getUTCFullYear()} ${months[date.value.getUTCMonth()]} ${date.value.getUTCDate()}`)
</script>

<template>
	<p class="VPLastUpdated">
		{{ theme.lastUpdated?.text || 'Last updated' }}:
		<time :datetime="iso">{{ said }}</time>
	</p>
</template>

<style scoped>
/* VitePress's own line heights for the line, so it sits beside the edit link as before, at the quotation's setting: 14px at the prose's weight in the secondary ink, which style.css gives the edit link too */
.VPLastUpdated {
	line-height: 24px;
	font-size: 14px;
	font-weight: 400;
	color: var(--vp-c-text-2);
}
@media (min-width: 640px) {
	.VPLastUpdated {
		line-height: 32px;
	}
}
</style>
