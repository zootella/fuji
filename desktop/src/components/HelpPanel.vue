<script setup>//the help panel: every key and mouse action the views and the shell handle, as a new user first sees them

//the text and its look; the shell decides where it sits and when it is up, over every view. A change to a key or a mouse action is a change here too

import {backspaceCloses} from './library.js'
import {brandName} from '../brand.js'

const closeKeys = backspaceCloses ? 'Esc|Backspace' : 'Esc'//backspace only where its key says Backspace, which library.js explains

</script>
<template>

<!-- a pre because it is the one element whose text vue's compiler leaves alone: in any other, the runs of spaces that line up the two columns would condense to one. The huds' smoky plastic from index.css, and no clicks or selection, so a click, a drag or the wheel goes through to the view beneath, which is still the thing the user is using -->
<pre class="myHud pointer-events-none select-none">
                                      {{brandName}}

                                 H    [H]elp
                                 I    [I]nformation
                                 S    [S]ettings, from the contact sheet

double click                          Back to the contact sheet
double click a thumbnail              Its picture, full screen
{{closeKeys.padStart(34)}}    Close the window
mouse wheel    PgDn|Up|Ctrl+arrows    Flip through images in order

drag                        arrows    Pan
                             space    Center image large in frame
                                 F    [F]it image within frame
                                 W    Size [W]idth to frame

Ctrl + mouse wheel             +|-    Zoom around frame center
right drag up|down                    Zoom around drag start
                               1-6    Exactly 1-6 CSS pixels per picture pixel
                                 R    [R]asterized or Smooth

                                 G    Gamma on and off
Shift + mouse wheel      Shift +|-    Gamma more and less
Shift + right drag up|down            Smooth gamma
</pre>

</template>
