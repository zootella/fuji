import tauriConfiguration from '../src-tauri/tauri.conf.json' with {type: 'json'}//the standard way to import json as a module, which vite and node both read
import cargoManifest from '../src-tauri/Cargo.toml?raw'//the file as text, which vite hands over for any import marked raw
import {parse as parseToml} from 'smol-toml'

/*
The product's two names, read when vite builds the page from the two files that set them, which are the files every build of the application already reads. The section The two names in the README at the root is the short version, for somebody renaming fuji or making a fork of it their own.

brandName is the name people read, Fuji, productName in tauri.conf.json. It goes wherever a person reads a name: the window's title and the Mac's application menu, every sentence on the page and in the comments of fuji.toml, the Mac's Fuji.app and its dmg, the Windows installer, Start menu entry and uninstall entry, and the names Windows shows beside a file type, Fuji.webp among them.

brandStem is the stem of the executable's name, fuji of fuji.exe, which is the crate's name in Cargo.toml, since Cargo names the executable from it. Stem is the word the languages use for a file name without its extension, Rust's file_stem and Python's stem. It goes wherever a file system or a program matches a name: the executable, fuji.toml, the fuji-temp folder the log is written into, and the published installer names, fuji.dmg, fuji.exe and the four linux packages. It is never a setting's value: the fonts setting says bundled for the two faces fuji carries, so that fuji.toml and the stylesheet name no product.

Nothing derives one from the other. Fuji and fuji differ only in case, but a product called Candy Crush may be candycrush or candy-crush inside, and only whoever names it knows which, so each is read from its own file. The identifier, app.fujidesktop.Fuji, is a third name with jobs of its own, the Mac's bundle identity, the folder WebView2 keeps its data in and the flatpak's identity, and it changes with the other two.

Rust reads the same two from its package info, name and crate_name, which tauri fills from the same files at compile time. The build scripts, dmg.js, win-setup.js, scripts.js at the root and linux/build.js, read the two files themselves, and win-setup.js compiles both into the Windows setup program. Two places write brandStem out, because nothing there can read it, and each says so: the Start tile's manifest, which tauri.conf.json's resources copy beside the executable under its name, since that is the only way Windows finds it, and the library's name in Cargo.toml, which main.rs calls.

Two addresses go with the two names, and the page keeps both bare, with no scheme, writing https:// in front where it links. urlHome is the site as a person says it, fujidesktop.app, the host of bundle.homepage in tauri.conf.json, which the installers already carry in their package information. urlHelp, fujidesktop.app/help, is where Fuji Help in the Help menu sends a person: a page on the site that forwards to whichever page help lives on. A copy of fuji in the field may never update, so it carries this one address for good, and the site decides where it leads, by changing that page and uploading. A fork whose help lives somewhere else writes its own address here.
*/

export const brandName        = tauriConfiguration.productName            //the name people read, Fuji
export const brandStem        = parseToml(cargoManifest).package.name     //the name files carry, fuji; read from Cargo.toml, and derived from nothing
export const brandDescription = tauriConfiguration.bundle.shortDescription//the one line the installers and the settings app show beside the name
export const brandVersion     = tauriConfiguration.version                //the version the installers carry and the settings panel's About shows, the one number fuji says about itself

export const urlHome = new URL(tauriConfiguration.bundle.homepage).host//the site, fujidesktop.app, as the installers name it with the scheme and the trailing slash left off
export const urlHelp = `${urlHome}/help`                               //where Fuji Help goes, fujidesktop.app/help, which the site forwards to wherever help lives
