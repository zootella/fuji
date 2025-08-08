<script setup>

</script>
<template>
<div class="min-h-screen p-[50px] myMarble">

<p>some text above</p>

<div class="w-[400px] h-[400px] border border-black bg-black myShadow"></div>

<p class="myEtched">some text below</p>

<p class="myVisible">another line of some more text</p>

</div>
</template>
<style scoped>

.myShadow {
	box-shadow: 6px 6px 12px rgba(0,0,0,0.5);
}
.myEtched {
	color: rgba(64,64,64,0.5);
	text-shadow: -1px -1px 0 rgba(0,0,0,1.0);

}
.myVisible {
  /* lighter semi-transparent text */
  color: rgba(64,64,64,0.5);
  /* x-offset: 0, y-offset: 0, blur: 6px, color: dark spray */
  text-shadow: 0 0 3px rgba(0,0,0,1.0);
}

.myMarble {
  background-color: #000;
  background-image: url("data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='100'%20height='100'%3E%3Cfilter%20id='noise'%3E%3CfeTurbulence%20type='fractalNoise'%20baseFrequency='0.6'%20numOctaves='2'%20stitchTiles='stitch'/%3E%3C/filter%3E%3Crect%20width='100%25'%20height='100%25'%20filter='url(%23noise)'%20opacity='0.12'/%3E%3C/svg%3E");
  background-repeat: repeat;
}

</style>
