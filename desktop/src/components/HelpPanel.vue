<script setup>//the help panel: every key and mouse action the table and the shell handle, as a new user first sees them

//the text and its look; the shell decides where it sits and when it is up, over every view. A change to a key or a mouse action is a change here too

</script>
<template>

<!-- a pre because it is the one element whose text vue's compiler leaves alone: in any other, the runs of spaces that line up the two columns would condense to one -->
<pre class="myHelp">
                                      Fuji

                                 H    [H]elp
                                 I    [I]nformation

double click                   Esc    Full screen
mouse wheel    PgDn|Up|Ctrl+arrows    Flip through images in order

drag                        arrows    Pan
                             space    Center image large in frame
                                 F    [F]it image within frame
                                 W    Size [W]idth to frame

Ctrl + mouse wheel             +|-    Zoom around frame center
Shift + drag up|down                  Zoom around drag start
                               1-6    Set 1x-6x raster to CSS pixels

                                 G    Gamma on and off
Shift + mouse wheel      Shift +|-    Gamma more and less
Shift + right drag up|down            Smooth gamma
</pre>

</template>
<style scoped>

.myHelp { /* the same smoky plastic as the table's huds in DiamondTable.vue, whose .myHud cannot reach this, since the shell draws it */
	color: rgba(255, 255, 255, 0.8);
	background-color: rgba(0, 0, 0, 0.4);
	padding: 0.1rem 0.4rem;
	font-size: 0.875rem;
	pointer-events: none; /* a click, a drag or the wheel goes through to the view beneath, which is still the thing the user is using */
	user-select: none;
}

</style>
