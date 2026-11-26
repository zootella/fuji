# Palette

Fuji's colors on one page, on each platform and in light and dark at once, beside the candidates weighed for them. It draws with Fuji's own stylesheet, faces, lucide drawings and components, so what it shows is what Fuji paints.

```
pnpm palette    from the side folder: serve the page on port 1430 and open it in the browser
```

Its tabs:

- **Theme**, every color `desktop/src/style.css` names, in light and in dark, with the stylesheet's own comment beside each, which says where the color came from.
- **Button Color**, palettes other people made, each shown whole, as candidates for the colors of the bar's drawings.
- **Crayon Boxes**, a box of distinct colors picked from each of those palettes by measurement, which `crayon.js` describes.
- **Accent Colors**, a selection, a toolbar and a menu in each platform's own look, a pane for each platform in each mode.
- **Brand Colors**, the brand colors first proposed, and the one Fuji took.

## How it works

`side`'s Vite configuration serves the page, reaching into `desktop/src` for the stylesheet and the components and into `desktop/public` for the faces, and has the page load one copy of Vue for its own files and Fuji's components alike.

`palette.js` reads the stylesheet as text and sets every color on the page's root again, light-dark() and all, so a color nothing in Fuji paints yet still shows, and a pane that sets light or dark draws every color in that mode. A pane that names a platform takes that platform's look the way a window does. Each caption gives a color as the engine paints it in its pane, not as the page supposes it.

A menu Fuji opens is drawn at the end of the page, outside every pane, where it would take the page's own mode and platform. So the panes draw copies of FujiBar's menu and of the right-click menu, `PaletteMenu.vue` and `PaletteMockMenu.vue`, whose rules are copied from those components and change when they do.

## What it found

It is where Fuji's colors were chosen: the face from the file managers' own lists, each platform's look for a selection and for a menu's row under the pointer, and the names that let each be set apart. The colors of the bar's drawings are still open, and the Button Color and Crayon Boxes tabs hold the candidates.

## What it needs

Nothing past `pnpm install` at the root. It builds nothing and ships nothing.
