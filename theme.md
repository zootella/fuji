# Theme

The colors fuji's interface is drawn in, light and dark. Begun 2026-10-06 on the Mac mini, at the start of moving the contact sheet off pure black and pure white. Three colors are in the code. `--color-paper` in `index.css` is the one background behind every view, `--color-dot` is the table's dots on it, and `--color-quiet` is the caption under each thumbnail on the sheet. Everything else in `index.css` is still the palette its own essay describes, Tailwind's neutral grays mirrored between the two themes.

## The decision

**Fuji copies Zed's default themes exactly: One Dark for dark, and One Light for light.** The user's choice, 2026-10-06. Zed follows the system's appearance and switches between these two, which is what fuji does too.

**Why Zed.** Fuji is about to grow the parts Zed already has: toolbars, buttons, a tree of folders beside the content, panels around it. An interface like that needs lines that divide it and panel backgrounds that tell one region from the next, so the whole is usable at a glance — and it needs all of that to step back, so the content is what the user looks at. In Zed the content is code; in fuji it is pictures. Zed's visual layout is close to the one fuji is heading for, and its themes get this balance right, so fuji takes them as they are rather than tuning a palette of its own.

**The grays are slightly blue, and that is accepted.** Zed's grays, like Tailwind's `gray` scale, lean cool: One Dark's editor background `#282c33` has more blue in it than red. That is the slate look of One Dark, and it is no mismatch for fuji, whose brand color is already its own shade of cyan. It does replace the rule `index.css` states today, that every gray is neutral and that light mirrors dark across the scale; One Light is its own theme, not a mirror of One Dark.

## The colors

Read from Zed's theme file, `assets/themes/one/one.json`, on `main` at commit `a3f6ef2` (2026-07-17). Zed writes every color as eight hex digits, the last two alpha; every one below is opaque, `ff`, except where the table gives the alpha.

The key is Zed's own name for each role, so a later reader can check it against the file.

**Backgrounds, from the content outward**

| Zed's key | One Dark | One Light | What it is in Zed |
|---|---|---|---|
| `editor.background` | `#282c33` | `#fafafa` | the content area; also `toolbar.background`, `tab.active_background` and `editor.gutter.background` |
| `panel.background` | `#2f343e` | `#ebebec` | the panels beside the content; also `surface.background`, `elevated_surface.background`, `tab_bar.background` and `tab.inactive_background` |
| `background` | `#3b414d` | `#dcdcdd` | the window behind everything; also `title_bar.background` and `status_bar.background` |

**Lines**

| Zed's key | One Dark | One Light | What it is in Zed |
|---|---|---|---|
| `border` | `#464b57` | `#c9c9ca` | the line that divides regions |
| `border.variant` | `#363c46` | `#dfdfe0` | a quieter line, within a region |
| `border.focused` | `#47679e` | `#7d82e8` | around what has the keyboard |
| `border.selected` | `#293b5b` | `#cbcdf6` | around what is selected |
| `border.disabled` | `#414754` | `#d3d3d4` | around what cannot be used |

**Controls**

| Zed's key | One Dark | One Light | What it is in Zed |
|---|---|---|---|
| `element.background` | `#2e343e` | `#ebebec` | a button at rest |
| `element.hover` | `#363c46` | `#dfdfe0` | a button under the pointer; the same as `ghost_element.hover`, for a button with no background of its own |
| `element.active` | `#454a56` | `#cacaca` | a button being pressed; the same as `element.selected` and `ghost_element.selected` |
| `editor.active_line.background` | `#2f343e`, alpha `bf` | `#ebebec`, alpha `bf` | the line the cursor is on, a band over the content |
| `drop_target.background` | `#838994`, alpha `80` | `#7e8087`, alpha `80` | where a drag would land |
| `scrollbar.thumb.background` | `#c8ccd4`, alpha `4c` | `#383a41`, alpha `4c` | the scrollbar's thumb, over a transparent track |

**Text and icons**

| Zed's key | One Dark | One Light | What it is in Zed |
|---|---|---|---|
| `text` | `#dce0e5` | `#242529` | ordinary text; the same as `icon` |
| `text.muted` | `#a9afbc` | `#58585a` | text that steps back; the same as `icon.muted` |
| `text.placeholder` | `#878a98` | `#7e8086` | a hint in an empty field; the same as `text.disabled` |
| `text.accent` | `#74ade8` | `#5c78e2` | a link, or text that stands out |

These are all the session read, and they are enough for the sheet, the toolbars and the tree. The file has many more keys, for syntax, the terminal and version control, and none of them is fuji's business yet.

## How fuji's parts map onto Zed's

A first reading, for the user to confirm part by part as each one moves.

- **Every view stands on one background, the content area's**, `editor.background` in dark, `#282c33`. In light it is pure white, `#ffffff`, rather than One Light's `#fafafa`: the user's choice, 2026-10-06, and the one place fuji departs from copying Zed exactly. Built as `--color-paper`, which the sheet, the settings panel, the table's ground, the preview and the fullscreen curtain all use, so moving between views never changes the color behind them. The user's decision, 2026-10-06, after a first pass gave the sheet a color of its own.
- **The settings panel, and a tree or a panel to come**, are `panel.background`.
- **A toolbar** is `editor.background` in Zed, the same as the content it sits over, divided from it by `border` rather than by a change of color. Nothing in fuji is that case today.
- **The table's ground is that same paper, with its dots a step off it**: `#363c46` in dark, One Dark's quiet line, `border.variant`, and `#ebebec` in light, One Light's panel gray. Their size and spacing, and where they sit against the card, are the table's own geometry and did not change.
- **The caption under a thumbnail is `text.placeholder`**, `#878a98` in dark and `#7e8086` in light, both of its lines, as `--color-quiet`. The user's choice, 2026-10-06, a step quieter than `text.muted`, which was tried first, and the bottom of Zed's three steps of text; Zed means it for hints and disabled text, so fuji's name for it is its own.
- **The line right around a picture is outside the themes**: pure white `#ffffff` in light and pure black `#000000` in dark, full stop, as `--color-frame`. The user's rule, 2026-10-06, replacing the brand cyan around thumbnails and paper around the table's card. Two CSS pixels and rounded at 4 around a thumbnail on the sheet, one CSS pixel and square around the card on the table, both outlines drawn outside the picture.

## Open

- **The shadow's colors.** Zed's theme file gives the backgrounds and lines but the shadow under a thumbnail is fuji's own: a black at some strength in each theme, chosen by eye on `#282c33` and on the light background. While it is tuned it is pure red, in `.myTile` in `TestFlow.vue`.
- **The settings panel is paper, not a panel gray.** Zed would draw a panel in `panel.background`, but the settings panel takes the whole window in place of the sheet, so it stands on the same paper as every other view. When fuji has a panel beside content, a tree or an inspector, that one is `panel.background`.
- **The whole palette, later.** Every token in `index.css` — ink, strong, faint, surface, line, edge and the rest — has a Zed role it would become. Moving them all at once is a larger change than the sheet needs; they move as each part of fuji is restyled, and the essay in `index.css` is rewritten when the last of the neutral mirror is gone.
