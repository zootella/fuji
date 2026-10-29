# Theme

## The decision

**Every gray fuji paints is Zed's, from its default themes, One Light for light and One Dark for dark.** The user's choice, begun 2026-10-06 with the paper and the captions, and taken to every gray on 2026-10-29. Zed follows the system's appearance and switches between these two, which is what fuji does too.

**Why Zed.** Fuji is growing the parts Zed already has: toolbars, buttons, a tree of folders beside the content, panels around it. An interface like that needs lines that divide it and panel backgrounds that tell one region from the next, so the whole is usable at a glance, and it needs all of that to step back, so the content is what the user looks at. In Zed the content is code; in fuji it is pictures. Zed's visual layout is close to the one fuji is heading for, and its themes get this balance right, so fuji takes their grays as they are rather than tuning a palette of its own.

**The grays lean slightly blue, and that is accepted.** One Dark's editor background, `#282c33`, has more blue in it than red. That is the slate look of One Dark, and it is no mismatch for fuji, whose brand color is its own shade of cyan. One Light is its own theme, not a mirror of One Dark.

**Not every color is Zed's.** A color copied on purpose from a platform keeps the platform's value, like the Mac's selected rows, rather than moving to the nearest gray of Zed's. And fuji's own colors are never gray: the warning's orange and amber, the icon's cyan, the white and black around a picture, and the shadows. The palette's essay in `style.css` says how each kind is named.

## Zed's keys for the parts to come

Read from Zed's theme file, `assets/themes/one/one.json`, on `main` at commit `a3f6ef2` (2026-07-17); light first, then dark, and opaque unless an alpha is given. These are the ones fuji does not paint yet and the toolbar, the tree and the buttons will reach for.

- **`background`**, `#dcdcdd` and `#3b414d`: the window behind everything, and Zed's title bar and status bar.
- **`panel.background`**, `#ebebec` and `#2f343e`: a panel beside the content.
- **`border.focused`**, `#7d82e8` and `#47679e`: around what has the keyboard.
- **`border.selected`**, `#cbcdf6` and `#293b5b`: around what is selected.
- **`border.disabled`**, `#d3d3d4` and `#414754`: around what cannot be used.
- **`element.background`**, `#ebebec` and `#2e343e`: a button at rest.
- **`element.hover`**, `#dfdfe0` and `#363c46`: a button under the pointer, the same as `ghost_element.hover` for a button with no background of its own.
- **`element.active`**, `#cacaca` and `#454a56`: a button being pressed, the same as `element.selected`.
- **`text.accent`**, `#5c78e2` and `#74ade8`: a link, or text that stands out.
- **`scrollbar.thumb.background`**, `#383a41` and `#c8ccd4`, alpha `4c`: a scrollbar's thumb over a transparent track.
- **`drop_target.background`**, `#7e8087` and `#838994`, alpha `80`: where a drag would land.

## How the parts to come map onto Zed's

- **A toolbar** is `editor.background` in Zed, the same as the content it sits over, divided from it by `border` rather than by a change of color.
- **A panel beside the content**, a tree or an inspector, is `panel.background`.
- **The settings panel stays on the paper**, `editor.background`, since it takes the whole window in place of the sheet, as every other view does.
