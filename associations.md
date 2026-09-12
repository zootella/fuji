# File associations

How fuji becomes an application the operating system will hand a picture to. What each platform actually offers, what Tauri writes on fuji's behalf, and the plan for the first pass.

The research came first and is kept below, because every decision in the plan is a consequence of one of those findings and the reasons go stale faster than the rules do.

**The research is read rather than measured, and the macOS half is now run.** The rules below come from Apple's and Microsoft's documentation, from Tauri's bundler source, and from the registry macros Tauri's installer inserts, in September 2026 — unusual for a document here and deliberate, because the subject is mostly other people's rules and the rules are written down. What has since been run on hardware is the macOS pass, on the Mac mini on 2026-09-12, and the open section at the end says what it found. Nothing on Windows has been run at all.

## Where this stands

**The first pass is built and works, and its last step is not fit for a human.**

Built and verified on the Mac mini: the ten types declared in a hand-written `Info.plist` that Tauri merges into the bundle; LaunchServices recording every one of them at rank Alternate; a double-click after *Change All* launching fuji onto that picture with its folder behind it and flipping and the `c` key working from the first frame; and a second double-click while fuji runs turning the same window to the new picture.

Built and not run anywhere: the whole Windows half — the runtime registration in `associate.rs`, the command-line path in `open.rs`. It compiles against the Windows target and that is all that can be said for it from here.

Not built: the uninstall hook, document icons, single instance, and any way for a user to discover or use any of this from inside fuji.

**The concession, stated plainly, because a later reader will otherwise assume this shipped in a usable state.** Fuji registers correctly and a person cannot reasonably be expected to find how to use it. Making fuji the default on macOS means Get Info, then expanding a collapsed *Open with:* section, then a small button reading *Change All…* — while the Open With submenu, which is the obvious place and the one a user will actually open, offers no way to set a default at all. The route took the person who had just written this document several tries to find. Nothing fuji does from outside Finder improves it. So the honest description of the first pass is that it works and that nobody will find it, which is tolerable only while the only user is the one who built it. The settings panel in the second pass is what fixes this, and that is now its reason rather than the milder one recorded below.

## The two questions, which are not the same question

**Being available is one thing and being the default is another, and every platform separates them.**

*Available* means fuji appears in the list when the user right-clicks a file and asks what could open it — Finder's Open With submenu, Explorer's Open with. It is a claim about capability, it costs nobody anything, and no platform has ever restricted it.

*Default* means a double-click goes to fuji without the user saying so again. It is a claim on the user's attention, it is what applications spent the nineties stealing from each other, and it is the thing every platform has since locked down.

**The shape the user has asked for is to request the first and never take the second,** leaving the second to the user, once, in the operating system's own interface. Everything below is read in that light.

## macOS

**Registration is declarative and lives in the bundle.** `CFBundleDocumentTypes` in `Info.plist` is an array, one entry per type fuji says it can open, and LaunchServices reads it when the application is first seen in a folder it watches. Nothing is registered at runtime and no installer writes anything: the `.app` carries its own claims, and dragging it to Applications is what publishes them.

The keys that matter:

    CFBundleTypeExtensions    the extensions, without dots
    CFBundleTypeName          a display name for the type
    CFBundleTypeRole          Editor, Viewer, Shell, QLGenerator, or None
    LSHandlerRank             Owner, Default, Alternate, or None
    CFBundleTypeIconFile      the icon shown on documents of this type

**`LSHandlerRank` is precisely the politeness knob.** `Owner` says this application owns the type. `Alternate` says it can open the type and is not claiming it. `None` says never offer it. The rank orders the candidates LaunchServices considers when there is no user choice on record; it does not override a choice the user has made.

**`Viewer` and `Alternate` together would say:** fuji can show one of these, it will not change it, and it is not asking to be first. That pairing is the polite end of what the keys can express.

**Setting the default programmatically is possible.** `NSWorkspace.setDefaultApplication(at:toOpen:)`, macOS 12 and later, replaces the deprecated `LSSetDefaultRoleHandlerForContentType`. For a document type it succeeds silently — the confirmation sheet Apple added in Monterey is for URL schemes, which is to say for the default browser, and a content type does not raise it. So on the Mac the restraint would have to be fuji's own; the system will not enforce it. A menu item that asks first and then makes this one call is the honest shape, and it is unbuilt.

**The user's own route, needing nothing from fuji, is Get Info.** Select a `.webp`, ⌘I, expand *Open with*, choose the application, then *Change All…*, which asks for confirmation and applies to every file of that type. That is the one-time action available to someone who wants it, whatever fuji does or does not build — and the section above is honest about how findable it is.

**The popup above that button sets one file, not the type, and that inversion is inherited rather than designed.** Classic Mac OS gave every file a type code and a creator code, and the creator code named the application that made it; double-clicking a document opened it in that application rather than in any system-wide default, so a JPEG from Photoshop and a JPEG from GraphicConverter opened in different places on the same machine. Files carried their provenance and that was the point. Mac OS X carried the behaviour into LaunchServices, extensions and then UTIs took over the typing, and creator codes were ignored by around 10.6 — but the per-file binding survived as the thing the popup does, with the type-wide choice demoted to a button beside it. So the common case takes the extra step and the rare case is the default, which reads as bad design and is really an old model's priorities left standing after the model was replaced. The rare case is still real — one text file that should open in an editor when the rest go to TextEdit — but it is rare, and it is what the interface puts first.

## Windows

**There are about five mechanisms with a claim on this, and only two of them matter.** Listing them all is the fastest way to see why.

1. **`HKEY_CLASSES_ROOT\.ext` default value → a ProgID.** The original mechanism, from the nineties. Whoever writes here last wins, which is exactly the problem that produced everything after it. **It was never removed and it still works**, as the fallback: when nothing below overrules it, this is the answer. `HKCU\Software\Classes` shadows `HKLM\Software\Classes` in the merged `HKEY_CLASSES_ROOT` view, so a per-user installer writing here outranks a machine-wide registration.
2. **`OpenWithProgids` and `Applications\app.exe\SupportedTypes`.** The polite additions. They put an application in the Open with list for an extension and claim no default at all. Microsoft's documentation says `OpenWithProgids` is preferred over the older `OpenWithList`, which was for pre-XP executables.
3. **`UserChoice`, under `HKCU\…\Explorer\FileExts\.ext`.** Since Windows 8 this is the actual answer to "what opens this file", and it beats everything in `Software\Classes`. The ProgID in it is validated against an undocumented per-user, per-extension hash, so an application cannot write it — a wrong hash makes Windows ignore the whole key. Since the February 2024 update a kernel driver, UCPD.sys, also blocks writes for `http`, `https`, and `.pdf` specifically.
4. **`RegisteredApplications` plus a `Capabilities` block.** How an application announces itself to the Settings app so it appears by name in Default apps with its list of types. It sets nothing; it makes the user's choice possible in one place.
5. **`SHOpenWithDialog`, and Windows' own "How do you want to open this file?" prompt.** The shell's own dialog. Its registration flags — `OAIF_ALLOW_REGISTRATION`, `OAIF_FORCE_REGISTRATION`, `OAIF_HIDE_REGISTRATION` — are ignored as of Windows 10, so calling it can no longer change a default either.

**So: there is no supported way for an application to make itself the default, and there has not been one since Windows 8.** Every tool that does it, `SetUserFTA` being the well-known one, works by reproducing the hash Microsoft has not published.

**What is left is a deep link into Settings.** With `RegisteredApplications` and `Capabilities` written, `ms-settings:defaultapps?registeredAppUser=Fuji` opens the Default apps page on fuji's own entry, where the user clicks. Windows 11 21H2 or 22H2 with the April 2023 cumulative update or later. It is one click away from where the user already is, and it is Microsoft's answer to the question fuji is asking.

**There is a modern API, and reading what it does settles the whole Windows design.** `ActivationRegistrationManager.RegisterForFileTypeActivation` in the Windows App SDK registers file types at runtime for an *unpackaged* application — it throws if the process is packaged, so it exists for exactly the kind of application fuji is. Its implementation is open, in `dev/AppLifecycle/Association.cpp`, and it does four things: writes a ProgID with a display name and a logo, writes the application's own key with a `Capabilities` block and adds it to `RegisteredApplications`, adds the ProgID to `OpenWithProgids` under each extension, and calls `SHChangeNotify(SHCNE_ASSOCCHANGED)` so the shell notices. It writes to `HKEY_CURRENT_USER`.

**What it does not do is set the extension's default value.** Microsoft's own current API registers politely and never takes a type. That is the strongest available evidence for what the supported shape is, because it is not documentation about intent — it is the code.

**Which means the modern way is fewer registry writes, not more, and fuji does not need the Windows App SDK to do it.** The API is a wrapper over about half a dozen values and one shell notification, all of which the `windows` crate fuji already depends on can do directly. Taking the SDK itself would mean its runtime deployed alongside a Tauri application, which is a large dependency for a thin wrapper.

**Windows does the asking, which is the part that makes this work without fuji nagging.** Microsoft's guidance says plainly: "Windows will automatically prompt the user when they open a file or link type when a new app is installed that registered for that file or link type." So an application registers, and the next time the user opens one of those files Windows itself offers the new choice. The application never has to ask for anything. The same page's best practices are: use the `ms-settings:defaultapps` deep link, prompt contextually rather than at startup, respect the user's choice, avoid repeated notifications, and only register for a type the application really will handle every launch of.

**And it states the rule that closes the subject:** "Windows does not allow programmatic changes to default apps without user interaction in system UI… Registry-based changes are not supported for apps." That is about the *default*, not about registration — registering is supported and is what the API above does.

**The Edge banner is the counter-example and the reason the door is shut.** "Security Warning: Microsoft Edge is Not Your Default Browser" is what an application does when it checks its own status at startup and has a way to act on the answer. An application that never checks can never write that sentence.

**The regulators changed the Settings page, not the API.** The Digital Markets Act work Microsoft shipped through 2025 makes the EEA "Set default" button set every type a browser registers at once, allows `.pdf` among them, and restricts Edge to prompting only when Edge itself is open. It is a better user interface for a choice the user was always making. None of it opens an application-facing route, and nothing in it applies to an application fuji's size.

## What Tauri does

**`bundle.fileAssociations` in `tauri.conf.json` is one list that produces different things per platform.** Fields: `ext` (required), `name`, `description`, `role`, `rank`, `mimeType`. `role` maps to `CFBundleTypeRole` and `rank` to `LSHandlerRank`; `rank` arrived in tauri-cli 2.5.0 and tauri-bundler 2.4.0, and fuji's CLI is 2.11.4, so it is available.

**On macOS it writes `CFBundleDocumentTypes` and that is the whole of it.** Correct, and the platform's own mechanism. It writes no `CFBundleTypeIconFile`, which is the icon question below, and it declares by extension rather than by `LSItemContentTypes` UTI. Extensions work.

**On Windows it does the nineties thing.** The NSIS template inserts a community macro, `APP_ASSOCIATE` from `FileAssociation.nsh`, which writes:

    Software\Classes\.webp        (default) = <the ProgID>        ← takes the extension
    Software\Classes\.webp        <ProgID>_backup = <what was there before>
    Software\Classes\<ProgID>     (default) = <description>
    Software\Classes\<ProgID>\DefaultIcon          = fuji.exe,0
    Software\Classes\<ProgID>\shell\open           = "Open with Fuji"
    Software\Classes\<ProgID>\shell\open\command   = fuji.exe "%1"

The first line is mechanism 1 above: it claims the extension. Tauri writes no `OpenWithProgids`, no `SupportedTypes`, and no `RegisteredApplications` or `Capabilities` — checked by reading `installer.nsi`, which contains none of those words. Under a per-user install the whole block goes to `HKCU\Software\Classes`, so it reaches one account rather than the machine.

**That write is effective exactly when nothing has claimed the type, which is the case that matters.** It is tempting to wave it away on the grounds that `UserChoice` outranks it, and that is the wrong conclusion: `UserChoice` outranks it *when there is one*. Microsoft layered the newer mechanisms on top of this one rather than replacing it, because eternal compatibility is the platform's prime directive and the installed base writes registry keys this way. There is no stable list of which extensions are enforced — it varies by Windows release, edition, policy, and which component is asking — so the only way to know for a given extension on a given machine is to look. What follows is the part that matters for fuji: the extensions most worth taking are the ones with no incumbent, and those are precisely the ones this write takes, silently, at install time, without asking.

**Set `name` or the ProgID is the bare extension.** Tauri uses `name` if given and the extension if not, so an association left unnamed registers a global class called `webp`. The convention, stated in the macro's own comments, is `Application.Type` — `Fuji.webp`.

**Linux gets `MimeType=` in the `.desktop` file** and nothing else, which is the correct and polite thing there. Linux is out of scope for this round anyway.

## How the file reaches fuji

**The platforms differ completely, and only one of them has an event.**

**macOS: `RunEvent::Opened { urls }`.** Tauri surfaces the Apple event, at launch and again every time the user opens another file while fuji is running. At launch it can arrive before the page exists, so it cannot simply be forwarded — it has to be held until something asks.

**Windows: a command line argument, in a brand new process.** Explorer runs `fuji.exe "C:\path\to\picture.webp"`, and it does that every time, whether or not fuji is already running. There is no event and nothing to listen for; `std::env::args()` is the whole mechanism.

**One shape covers both platforms: hold a list in Rust and let the page drain it.** Something filled from `argv` at startup on Windows and from `RunEvent::Opened` on the Mac, handed over and emptied when the page first asks. The page then has one path through this rather than two, and the launch-time and while-running cases differ only in when the list gets filled. Unwritten.

**A second double-click while fuji is running starts a second fuji on Windows, and that is the first pass's behaviour on purpose.** The alternative, `tauri-plugin-single-instance`, hands the new process's arguments to the running one and exits — which on today's fuji would mean the running window turning to the new picture and losing what it was showing. Two windows is closer to what the user wanted than one window that forgets. `instances.md` owns the real answer; this pass takes each platform's default behaviour and adds nothing.

## The plan

The first pass, planned on the Mac mini and not yet built. Every decision here traces to a finding above.

### What fuji declares

**The list is `imageTypes` — every extension fuji can show, and no other list anywhere.** Declaring is free and takes nothing, so there is nothing to choose between and no setting to hold a choice. When `thumbnail-open.md` settles HEIC or TIFF, they arrive here by being added there.

    .bmp    BMP Image
    .gif    GIF Image
    .jpg    JPG Image
    .jpeg   JPEG Image
    .jpe    JPE Image
    .jfif   JFIF Image
    .png    PNG Image
    .svg    SVG Image
    .avif   AVIF Image
    .webp   WebP Image

**The name is the extension, uppercased, plus Image — one per extension and never one per format.** Calling all four JPEG variants "JPEG Image" would be more correct and is what Finder's Kind column does, and it is wrong here for a practical reason: the Type column in Windows Explorer is the only way to sort a folder by extension. A user who wants to find the three `.jpe` files among a thousand `.jpg` clicks Type, and a shared name scatters them through the sort instead of grouping them. Correctness that defeats the one job the column has is not correctness.

**The names are short deliberately**, because that column is narrow. `WebP` keeps the format's own capitalisation rather than becoming `WEBP`; it is the only extension where the uppercase rule and the format's real name disagree, and there is only one of it, so nothing sorts differently either way.

**`imageTypes` grows a name and stays the single source.** It maps an extension to a MIME type today and becomes a map to `{mime, name}`, which touches `cache.js`, `SquareFlow.vue` and `listFolder`. Small, and better done once now than twice later — the settings panel in the second pass reads the same names.

### macOS: one hand-written file

**`src-tauri/Info.plist`, holding `CFBundleDocumentTypes` and nothing else.** Tauri merges it with the plist it generates, so everything it already writes — identifier, version, icon, minimum system — keeps coming from `tauri.conf.json`.

One dict per extension, ten of them, so the plist and the Windows registration are the same loop over the same list. A dict can carry several extensions, and grouping the four JPEG variants into one would be tidier — but then `CFBundleTypeName` is shared, and the name is per extension for the Explorer reason above. Keeping the two platforms parallel is worth more than three saved dictionaries.

    CFBundleTypeName         JPE Image
    CFBundleTypeExtensions   jpe
    CFBundleTypeRole         Viewer
    LSHandlerRank            Alternate

`CFBundleTypeName` barely shows on macOS — Finder's Kind column comes from the system's UTI description rather than from anything fuji declares — so this costs nothing there.

**Declared by extension rather than by UTI.** `LSItemContentTypes` with `public.jpeg` and friends is the modern form, but a mistyped UTI fails silently with nothing reported anywhere, and extensions are what Tauri has been generating all along. UTIs can come later if something needs them.

**`bundle.fileAssociations` is not used at all**, because it is one cross-platform list and using it makes the Windows installer take every extension in it. That is the single reason the plist is hand-written.

**The cost is a second copy of the list**, in the plist, which no code can read. A mismatch means fuji declaring a type it cannot show or showing one it never offered. Each list gets a comment pointing at the other; generating the plist from `imageTypes` with a small Node script is in the open list below.

### Windows: one command, at startup

**Per-extension ProgIDs — `Fuji.jpg`, `Fuji.png` — and not one shared `Fuji.image`.** Microsoft's own API computes a single ProgID per application, but it never needs per-type document icons, and `DefaultIcon` hangs off the ProgID. The deciding argument is that this is the choice that never needs undoing: split a shared ProgID later and every `UserChoice` a user has set still points at a ProgID fuji has stopped writing, which is a migration with no clean answer. Per-extension costs nothing now — it is the same loop either way.

For `.jpg`, under `HKEY_CURRENT_USER`:

    Software\Classes\Fuji.jpg                        (default)       = JPEG Image
    Software\Classes\Fuji.jpg\DefaultIcon            (default)       = <exe>,0
    Software\Classes\Fuji.jpg\shell\open\command     (default)       = "<exe>" "%1"
    Software\Classes\.jpg\OpenWithProgids            Fuji.jpg        = (empty)
    Software\Classes\Applications\fuji.exe           FriendlyAppName = Fuji
    Software\Classes\Applications\fuji.exe\shell\open\command       = "<exe>" "%1"
    Software\Classes\Applications\fuji.exe\SupportedTypes  .jpg     = (empty)
    Software\Fuji\Capabilities                       ApplicationName = Fuji
    Software\Fuji\Capabilities\FileAssociations      .jpg            = Fuji.jpg
    Software\RegisteredApplications                   Fuji            = Software\Fuji\Capabilities

then `SHChangeNotify(SHCNE_ASSOCCHANGED)`, once, and only if a write actually changed something.

**The one value never written is `Software\Classes\.jpg` itself.** That is the line that takes a type, it is the line Tauri's installer writes, and it is the line Microsoft's own modern API does not. Everything above makes fuji *available*; nothing above makes fuji *default*.

**It runs at every launch, from fuji rather than from the installer.** Idempotent, six writes an extension, and it self-heals if something has removed them. The installer writes nothing about file types at all.

**Except in a debug build, where it does not run.** The command paths come from `std::env::current_exe()`, so registering under `pnpm local` would point the user's registry at a debug binary in `target/debug` that moves and disappears. The whole feature is for an installed fuji, so a debug build registers nothing and says so in the log.

### Two Rust modules, both dumb

**`open.rs` — a file the operating system handed fuji.** One command, `open_files()`, which returns the paths and empties the list. The list is filled from `RunEvent::Opened` on macOS and from `std::env::args()` at startup on Windows. On macOS it also emits an event saying there is something to collect, because that case arrives while fuji is already running.

**`RunEvent::Opened` is `#[cfg]`-gated to macOS, iOS and Android, and its payload is `Vec<url::Url>`** — checked in tauri 2.11.5's `app.rs`, which is the version fuji builds against. The match arm in `lib.rs` needs the same `#[cfg(target_os = "macos")]` attribute or the Windows build will not compile, which is the kind of breakage that lands on whoever pulls next rather than on whoever wrote it. The URLs are `file://`, so `to_file_path()` is the conversion.

**`associate.rs` — what fuji has told the operating system it can open.** One command, `associate_register(types)`, taking the extensions with their ProgIDs and names and writing the block above. A no-op on macOS and Linux. It grows the query and set commands in the second pass and nothing else.

**Neither decides anything.** The page owns the list, composes the ProgID strings, and chooses when to call — which is the rule in `CLAUDE.md` about the two layers, and the reason the Windows registration is a page-driven call rather than something Rust does for itself at startup.

### What the page does

In `Shell.vue`'s existing startup order, four additions:

1. After the settings read, `openFiles()` — the paths the operating system handed fuji, usually none.
2. If there are any, show a table rather than whatever view fuji was last in, **without writing that back to `fuji.toml`**. Someone who double-clicked a picture asked to see that picture; they did not change their mind about where fuji opens.
3. After the reveal and after the active view's `start()`, hand the first path to that view's `onDrop`. A launch with a file is a drop that fuji was not running for, so it takes the path that already exists: `modelOpen()` lists the folder, applies the current sort and stands on the image, and flipping and the `c` key work from the first frame with nothing new written.
4. Register on Windows, after the reveal so it can never delay the window appearing.

**Every path coming in this way is forwardized**, in `Shell.vue`, at the same boundary where a dropped path already is. Windows hands over `C:\\Users\\...\\picture.webp` with backslashes, and everything downstream of that line assumes forward slashes. Missing it breaks the entire Windows half while leaving the Mac working, which is the worst shape for a cross-machine bug.

A listener for the macOS event does 1 and 3 again while fuji is running.

### Where fuji tells the user any of this, and the hole it leaves

**Nowhere, in this pass, and that is a change from the plan as first written.** The intention was the help HUD, since `h` already shows a centred block of plain text and that is lines in a string rather than new interface. Opening it found placeholder copy — "this HUD will likely be a card showing the user all the keyboard shortcuts" — rather than a help screen, and putting real instructions inside acknowledged filler makes both worse. So the HUD is left alone, and the discovery lines go in when help is designed: the Get Info and *Change All* route on macOS, and on Windows the Open with route and the Settings page. Until then fuji is silently available and the user has to know.

**That was too casual, and the smoke test is why.** Making fuji the default on macOS means Get Info, then expanding a collapsed *Open with:* section, then a small button labelled *Change All…*. It works, and it took the person who had just written this document several tries to find, with the Open With submenu — the obvious place — offering no way to set a default at all. The route is not hard once known and it is close to invisible before, and nothing fuji can do from outside Finder improves it. This is the argument for the settings panel in the second pass, and it is a stronger argument than the one recorded there, which was only that a user might reasonably start in the application.

**Fuji never checks whether it is the default.** Not at startup, not on a launch with a file, not anywhere in this pass. That single restraint is what keeps fuji from ever growing the banner that started this conversation.

### What the user experiences

**macOS.** Drag `Fuji.app` to Applications; nothing appears, because macOS never announces a new handler. From that moment fuji is in Finder's Open With for those ten types, before it has ever been run. Double-click still opens Preview. A user who wants fuji selects a file, ⌘I, *Open with* → Fuji, *Change All…*, and confirms the sheet macOS puts up. After that, a double-click launches fuji onto that picture with its folder behind it, and a second double-click while fuji is running turns the same window to the new picture.

**Windows.** Run the installer, which writes nothing about file types. Launch fuji once; it registers. From that moment fuji is in Explorer's Open with, and listed as *Fuji* in Settings under Default apps with all ten of its types. Double-click still opens whatever it opened before. A user who wants fuji has three routes: Windows' own prompt the next time they open one of those types, or Open with → Choose another app → Fuji with *Always use this app*, or Settings → Set defaults by app → Fuji → Manage, which is the whole list with a picker each. A second double-click while fuji is running starts a second fuji, which `instances.md` owns.

### Uninstalling

**One NSIS hook, and only for the uninstaller.** Fuji wrote those keys at runtime and the uninstaller knows nothing about them, so without this, uninstalling leaves fuji in Open with lists pointing at an executable that is gone. `NSIS_HOOK_PREUNINSTALL` deletes the ProgIDs, the `Applications\fuji.exe` key, the `Capabilities` block and the `RegisteredApplications` value, and the `OpenWithProgids` values it wrote — only keys fuji created. No install hook; this is the one place NSIS still earns one. macOS needs nothing, because trashing the `.app` takes its claims with it.

## The second pass

Not built here, and recorded now so the first pass can be checked against it. Agreed in conversation 2026-09-12.

**A File Extensions page in fuji's settings**, when fuji has settings, because a user who wants fuji to open JPEGs will reasonably start in fuji rather than in Finder. One table, both platforms, a row per extension: the extension, what opens it now, and an action.

**The action is the only thing that differs, and its label says which it is.** *Use Fuji* on macOS calls `NSWorkspace.setDefaultApplication`, which is supported, silent and per type. *Choose in Settings…* on Windows opens `ms-settings:defaultapps?registeredAppUser=Fuji`, because Windows does not let an application set a default at all, and the page it lands on is the same list with the system's own pickers — arguably better than what fuji could draw.

**Reading the current handler is supported on both** and is three small commands on `associate.rs`. Microsoft's guidance says explicitly that an application may query the current default; macOS answers through `NSWorkspace.urlForApplication(toOpen:)`.

**One wrinkle to expect on macOS:** that call wants a file URL rather than a type, and the type-shaped call, `urlsForApplications(toOpen:)`, returns the candidates rather than the winner. So "what opens `.jpg` right now" is the awkward direction on the Mac. Either a throwaway file per type or the deprecated `LSCopyDefaultApplicationURLForContentType`, which still works. An experiment for that pass.

**The rule that has to survive into it:** fuji queries only when the user is looking at the answer. Opening the File Extensions page asks a question and gets one; startup, a launch with a file, and every other moment ask nothing and display nothing. The query API and the Edge banner are the same call — only the trigger separates them, so the trigger is the rule.

## Icons

**The gap in the first pass, and the one worth naming loudly.** On Windows a ProgID's `DefaultIcon` is what Explorer draws on every file of that type, and the plan points it at fuji's own application icon. So the moment a user makes fuji the default for `.svg`, every `.svg` in a project folder wears the fuji logo — which is exactly the complaint that prompted this work, aimed at a different application. Fuji would be doing the same thing with a different picture.

**It is invisible until someone chooses fuji,** which is why the first pass can ship without it. It is not why it should stay that way.

**A document icon is a new artifact and belongs to `icon.md`,** which already owns how fuji's icons are made and what each platform expects: an `.icns` in the bundle's Resources named by `CFBundleTypeIconFile` on macOS, and something for `DefaultIcon` to point at on Windows. Per-extension ProgIDs mean a different icon per format is possible later without a migration.

**On macOS a document type with no `CFBundleTypeIconFile` gets a generic document icon.** But Finder draws pictures as Quick Look previews rather than as the handler's document icon, so the expectation — untested — is that these files keep looking like themselves on the Mac. Ten minutes on the Mac mini settles it.

## Open

**What the Windows 10 box has to answer, and the first work for the session over there.**

- Does Windows prompt the user the next time they open a registered type, after a *runtime* registration rather than an install-time one? Microsoft's guidance promises the prompt but is written around Windows 11 and around installation. If it does not fire, the help HUD carries all of the discovery and that is worth knowing before shipping.
- Does `ms-settings:defaultapps?registeredAppUser=Fuji` work on Windows 10, or only on 11? The parameter is documented for 11 with the April 2023 update. Plain `ms-settings:defaultapps` is the fallback.
- Does Explorer still draw thumbnails for a type fuji has been chosen for? A thumbnail handler hangs off a `ShellEx` key that can sit on the extension or on the ProgID. If it sits on the ProgID, a folder of pictures becomes a folder of identical fuji icons, which would change the plan rather than merely disappoint.
- What `Software\Classes\.webp` and the `FileExts` key hold before and after, which is the check that the registration did what this document says it does.

**What the Mac mini answered, 2026-09-12.** All of it works. Tauri merged the hand-written plist without disturbing its own keys, LaunchServices recorded every type, and a double-click after *Change All* reaches fuji with its folder behind it. Read out of `lsregister -dump`:

    claim id:     JPG Image
    rank:         Alternate
    roles:        Viewer
    bindings:     .jpg
    claimed UTIs: public.jpeg, public.png, public.svg-image, public.avif,
                  org.webmproject.webp, com.compuserve.gif, com.microsoft.bmp,
                  dyn.ah62d4rv4ge80y3xmq2 (.jfif)

**Declaring by extension is enough, which settles the question this document left open.** macOS synthesised the right system UTIs from bare `CFBundleTypeExtensions` — `public.jpeg`, `public.svg-image`, and the rest — with no `LSItemContentTypes` written anywhere. The caution about a mistyped UTI failing silently stands, and there is now no reason to take the risk.

**`.jfif` is the exception and gets a dynamic UTI,** because macOS has no system type for it. It works, but a synthesised type is its own type: *Change All* on a `.jpg` sets `public.jpeg` and does not cover `.jfif`, which has to be set separately. Nothing to fix, worth knowing.

**Two bundles claim these types on a development machine, and they share a bundle identifier.** The copy in `/Applications` and the copy under `target/release/bundle/macos/` are both registered, and LaunchServices stores the user's choice by identifier rather than by path, so it may launch either. Usually the installed one wins. If it ever picks the build directory, the next build replaces the binary underneath it.

**Generating `Info.plist` from `imageTypes`** with a small Node script, the way `release.js` set the precedent, so the list exists once. It needs `imageTypes` moved out of `library.js` into a module with no imports, since `library.js` pulls in Tauri APIs that Node cannot load.

**The macOS read wrinkle**, above, before the second pass depends on it.

**Linux**, which gets `MimeType=` in its `.desktop` file already and nothing tested after that.

**More extensions**, following `thumbnail-open.md` rather than leading it.
