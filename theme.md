# Theme

## The decision

**Every gray fuji paints is neutral, and its lightness is Zed's.** A gray named for its job, like `text` or `border`, is one of Zed's from its default themes, One Light for light and One Dark for dark, at that gray's own lightness in OKLab with its tint taken out, so its red, green and blue are one number. A numbered gray, `gray-50` to `gray-950` every 25, is Tailwind's neutral at that number, the same in light and dark. The user's choice: Zed's grays begun 2026-10-06 with the paper and the captions and taken to every gray on 2026-10-29, and made neutral on 2026-11-18. Zed follows the system's appearance and switches between its two themes, which is what fuji does too.

**Why Zed.** Fuji is growing the parts Zed already has: toolbars, buttons, a tree of folders beside the content, panels around it. An interface like that needs lines that divide it and panel backgrounds that tell one region from the next, so the whole is usable at a glance, and it needs all of that to step back, so the content is what the user looks at. In Zed the content is code; in fuji it is pictures. Zed's visual layout is close to the one fuji is heading for, and its themes get this balance right: calm grays that still read, and how far each line and panel stands from what it is drawn on. That balance is in each gray's lightness, so fuji takes the lightness exactly rather than tuning a palette of its own.

**Why neutral.** Zed's grays lean blue, faintly in One Light's text and plainly all through One Dark, whose editor background, `#282c33`, has more blue in it than red. That suits an editor, whose content is its own text and has to match only itself. Fuji's content is the user's pictures and video, and a gray beside a picture that leans any way shifts how the colors inside the picture read, so fuji takes Zed's lightness and leaves its tint behind, and a neutral fuji is the one least likely to look wrong beside whatever the user is viewing. Neutral also agrees with the platform around fuji, whose title bars and scrollbars are neutral. Measured when the change was made, every gray kept its lightness to within 0.16 in OKLab; One Light's grounds and lines moved by one in a single channel at most, and its text grays by up to six in blue, so light looks the same and dark loses its slate.

**Not every color is a gray.** A color copied on purpose from a platform keeps the platform's value, like the window buttons fuji draws on Windows and Linux, whose gray comes from Windows 10 and whose red from Windows 11, rather than moving to the nearest of fuji's grays. The accent is the user's. And fuji's own colors are never gray: the warning's orange and amber, the icon's cyan, the white and black around a picture, and the shadows. The palette's essay in `style.css` says how each kind is named.

## Zed's keys for the parts to come

Read from Zed's theme file, `assets/themes/one/one.json`, on `main` at commit `a3f6ef2` (2026-07-17); light first, then dark, and opaque unless an alpha is given. These are the ones fuji does not paint yet and the toolbar, the tree and the buttons will reach for. Each joins the palette as the grays already in it did, at its own lightness with its tint taken out and under a name for its job; the colors that are not gray, the focus ring's blue and the link's, are a question of their own.

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

## Not Zed all the way

Notes and a concern, 2026-11-14, not a plan, and not being resolved or decided now. The second and the last were settled on 2026-11-18, in the decision above.

- **Fuji may not stick with Zed.** The user is no longer certain of it. If the user likes another palette better, GNOME's for instance, fuji might switch to that.
- **Zed's tinted grays may not be right for fuji.** For a retro look, and to best showcase the user's media, neutral gray may be the right choice. There is already a clash with the browser's scrollbars around the tree and the list. Settled: fuji's grays are neutral, at Zed's lightness.
- **Leaving Zed would keep its values.** If fuji leaves Zed behind, it takes the values, and may map them onto Tailwind style numbers.
- **Numbers every 25, too**, which should be pretty easy. Settled: the numbered grays are every 25, from 50 to 950. A gray named for its job keeps Zed's lightness exactly and need not fall on one, and nudging one onto a notch would be a choice of its own.
