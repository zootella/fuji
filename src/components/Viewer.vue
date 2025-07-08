<script setup>

import {ref, onMounted} from 'vue'

const imageSrc = ref('')

onMounted(() => {

	const filePath = '/Users/kevin/Downloads/example.png'

	imageSrc.value = `file://${filePath}`
})

/*
i see script setup above, but below, should this be style scoped
can i use tailwind styles in here instead of vanilla css?
*/

</script>
<template>

<div class="image-container">
	<img :src="imageSrc" />
</div>

</template>
<style scoped>

.image-container {
	width: 100%;
	height: 100%;
	display: flex;
	background: black;
	justify-content: center;
	align-items: center;
}
.image-container img {
	max-width: 100%;
	max-height: 100%;
	object-fit: contain;
}

</style>
