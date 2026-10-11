# Apple

Fuji's Mac release through Apple: signed with a Developer ID, notarized, and published, from a waiting room of its own.

```
pnpm waiting-room here: build, notarize, hash and upload, a key each
pnpm notarize     in desktop: build and notarize straight through, without the waiting room
```

## Why this workspace exists

A browser marks every file it downloads, and Gatekeeper checks a marked app the first time it runs. Gatekeeper trusts what Apple vouches for, and Apple vouches twice. A Developer ID certificate, which Apple issues to a member of its developer program, says who signed the app. Notarization, Apple's service that examines a submitted build and returns a ticket, says Apple looked at it. An app with both meets the question every app from the Internet meets, with an Open button. `pnpm installer` in desktop signs Fuji with Fuji Desktop instead, a certificate of Fuji's own, and a downloaded copy of that meets "Apple could not verify Fuji is free of malware", with the way past it in System Settings.

Everything that depends on Apple's program lives here, and desktop needs none of it. `tauri.conf.json` still names Fuji Desktop, so a release that depends on no one is still `pnpm installer`, `pnpm hash` and `pnpm upload` away, and that is the fallback. If Apple revoked the certificate, Macs online would refuse every copy signed with it, installed ones included, and the next release would go out from desktop, with the download page describing the way past the dialog again. A lapsed membership is milder: copies already notarized keep working, and only new ones cannot be notarized. Each signature carries a secure timestamp, so a copy signed before the certificate expires stays valid after it does.

## The waiting room

A release spends most of its time here the way a visit to the DMV does, waiting for its number to be called and then going up for the next step.

```
apple waiting room

❯ 2026-Oct-12 Mon 0.1.1 5be20c7a  Notarizing .app... (12 minutes, 3 seconds)
  2026-Oct-10 Sat 0.1.0 9c0d1e2f  Uploaded

    .app    3d8a1f52-6c0e-4b7a-9e15-2f4c8b7d1a63
            sent 2026-Oct-12 Mon 2:04 PM · In Progress

b build · n notarize · h hash · u upload · ↑↓ choose · q quit
```

Each row is an application, newest first. An application is one build of Fuji, signed with the Developer ID and kept in a folder of its own under `applications/`, named for the day it was built, its version, and a tag: the first eight hex digits of the app's code hash, which is what Apple binds its ticket to. Under the rows are the selected application's submissions, as Apple's history has them, then when it was notarized and how long that took, and the dmg's sha256. A line at the bottom appears only while Apple has not answered yet, or when it cannot be asked.

Below those, **under way** lists every step running on this Mac, whichever terminal started it, with how long it has been going:

```
under way
    dmg      Fuji_0.1.1_aarch64.dmg  of the app of 2026-Oct-12 Mon 0.1.1 5be20c7a, stapled, by pnpm notarize (4 seconds)
```

That comes from the build log, which the waiting room looks at every second. Any new line in it, from a key here or a command in another terminal, makes the waiting room read the folders, Apple's history and the site again at once. It also reads them every half minute, for what happens with nothing on this Mac watching, like Apple finishing overnight.

**Nothing happens without a key, and one thing happens at a time.**

- **b** builds Fuji signed with the Developer ID and keeps the signed app as a new application at the top of the list. It took 24 seconds on the Mac mini on 2026-10-09, and takes longer after a stretch of `pnpm local`, since the release build shares nothing with the debug one. A build with nothing changed signs to the same code hash, so b keeps no second copy and selects the application it matches.
- **n** notarizes the selected application, from wherever it stands, and leaves its dmg in Tauri's bundle folder, where `pnpm installer` leaves its own.
- **h** hashes it. It puts the dmg in the bundle folder again, since another may have landed there after n, and runs the same hash as `pnpm hash` in desktop, which stages `desktop/release/fuji.dmg` and writes its sidecar, so staging from either place comes out the same. The application's version has to match `tauri.conf.json`, which is where hash reads the version it publishes.
- **u** uploads what h staged, the same upload as `pnpm upload` in desktop.
- **↑↓** choose an application, and **q** quits.

### Statuses

| Status | Means | n does |
|---|---|---|
| Built | signed, with nothing sent to Apple, and the clock stopped at how long b took | sends the app |
| Notarizing .app... | Apple is examining the app, and the clock is Apple's own | waits on it |
| Notarized .app | Apple accepted the app, and the dmg is not with Apple yet | carries on with the dmg |
| Notarizing .dmg... | Apple is examining the dmg, and the clock is Apple's own | waits on it |
| Notarized .dmg | Apple accepted the dmg, and it is not stapled and checked yet | staples it and checks |
| Notarized | the dmg carries its ticket, and Gatekeeper accepts the app and the dmg, with the clock stopped at how long that took from Apple receiving the app | puts the dmg in the bundle folder again |
| Hashed | the dmg staged in `desktop/release` is this one | |
| Uploaded | the dmg the site serves is this one | |
| Rejected | Apple turned down the app or the dmg | nothing, and b builds another |
| Unknown | Apple has not answered yet, or could not be asked, and the line at the bottom says which | |

Hashed and Uploaded are read from the sidecars, the one staged in `desktop/release` and the one live on the site, so they say what is true now. When a newer application goes up, the older one goes back to Notarized, with its upload kept in its log. Work on this Mac, a build, a dmg being made, a hash or an upload, shows under the rows as under way rather than as a status.

### What n does

The app goes to Apple first, then the dmg holding it. Apple's rule for a container is to notarize the outermost one and staple its ticket to it, and doing both leaves the app and the dmg each carrying a ticket of their own, so neither needs a network at first launch. Developers report Gatekeeper refusing a dmg that is signed and never notarized.

- **The app.** n zips it and sends it, then asks Apple about it every half minute until Apple answers.
- **The dmg.** n staples the app's ticket to it, wraps it in a dmg with `desktop/dmg.js`, the same layout `pnpm installer` makes, signs the dmg and sends it, then waits again.
- **The checks.** n staples the dmg's ticket to it and asks Gatekeeper what a downloaded copy will meet, writing every verdict to `verify.txt`. Each should say accepted, with source=Notarized Developer ID.
- **The delivery.** n leaves the dmg in `desktop/src-tauri/target/release/bundle/dmg/`, where `pnpm installer` leaves its own and `pnpm hash` looks. Whichever of the two made a dmg last is the one hash finds, and the loud warning at a dmg that is not notarized says when that was `pnpm installer`.

Each submission is named for its application, `Fuji 2026-Oct-12 Mon 0.1.1 5be20c7a.zip` and the same with `.dmg`, and **n always starts by reading Apple's history for the two whose names end with its tag.** Whatever it finds, it waits on or carries on from, and it never sends either part twice. Pressing n again is safe after a quit, after a failure, and on a row Apple is still examining.

Tauri can notarize on its own, but only the app, and only while one process waits for Apple's answer. So b gives Tauri the identity and not the API key, and Tauri signs and stops.

How long Apple takes varies. For the first release, on 2026-10-09, Apple took 51 minutes over the app and 25 seconds over the dmg. The second, on 2026-10-10, took 1 minute and 13 seconds from Apple receiving the app to the last check passing. A new team's first submissions can be held for hours or days.

### Quitting

Quitting never loses anything, and starts nothing new. A wait on Apple ends at once and Apple keeps examining. Come back later and the row shows Apple's own clock, or Notarized .app if Apple finished while the waiting room was closed, and n carries on from there. Work on this Mac finishes before the waiting room closes: a build, or the stretch between Apple's two answers, from stapling the app to sending the dmg, so a dmg is either with Apple or not yet made. Ctrl-C does the same with the screen already gone, and a second Ctrl-C ends it at once.

So while n waits on Apple, which holds the one job for as long as Apple takes, quitting and coming back is how to build or publish something else. A sleeping Mac pauses the asking and never Apple's examination, and the waiting room catches up when it wakes.

### From another terminal

`pnpm notarize` in desktop is b and then n, printing each step's lines as it starts and ends, with how long it took, and the waiting room can watch it without a key pressed: each step under way within a second of its starting, the new application's row once it is built, and its row following Apple from there. Stopped partway, `pnpm notarize` can simply run again. It builds, finds the same code already kept, and carries on with that application from wherever it stands. Like n, it ends with the dmg in the bundle folder, so `pnpm hash` and `pnpm upload` publish it from desktop, and running it again after a `pnpm installer` puts the notarized one back.

Different things at once are fine, and the same thing twice is not. Two n's on one application would each make a dmg in the same folder and send it, and two builds, b or `pnpm installer`, share the folder Tauri bundles in. So one n per application and one build at a time.

## The build log

`build-log.txt` at the root of the repository has a line for every step anything takes on this Mac, as it starts and as it ends: the waiting room's keys, `pnpm notarize`, and the desktop and Linux workspaces' `pnpm installer`, `pnpm hash` and `pnpm upload`. It is gitignored, so each machine keeps its own, and it stays readable after the scripts that wrote it have changed. Each line carries the commit the checkout was at, with a + when tracked files had changed since, so a hash in the log can be traced back to the source it came from. The essay atop `build-log.js`, beside `scripts.js` at the root, describes the format.

```
2026-10-11T15:26:37.522Z  81234    c1c95420+  send      started   88a2967d.app              -                                                                 as Fuji 2026-Oct-11 Sun 0.1.0 88a2967d.zip, by pnpm notarize
2026-10-11T15:26:41.880Z  81234    c1c95420+  send      finished  88a2967d.app              3e6a18ec-7934-40ae-82fa-80357ce8df23                              Apple's id for it, took 4.3 seconds
2026-10-11T15:26:41.884Z  81234    c1c95420+  apple     started   88a2967d.app              3e6a18ec-7934-40ae-82fa-80357ce8df23                              with Apple, by pnpm notarize
2026-10-11T15:27:12.010Z  81234    c1c95420+  apple     finished  88a2967d.app              Accepted                                                          Apple's answer, took 30.1 seconds
```

The apple workspace writes build, dmg, send, apple, check and deliver; `pnpm installer` writes dmg on the Mac and exe on Windows, the Linux workspace writes linux, and every workspace's `pnpm hash` and `pnpm upload` write hash and upload. A step on this Mac starts and ends in one process, sending a part among them. Apple's step starts when the send finishes and ends when Apple's answer is recorded, by whichever process learns it, so a quit in between leaves it started, which is true: Apple is still examining it. There is no step for a whole notarization, because a notarization can outlast a shutdown; its time runs from the app's apple started line to the check's finished one.

## Application folders

```
applications/2026-Oct-12 Mon 0.1.1 5be20c7a/
	Fuji.app                  the signed app, copied out of the bundle the moment b made it, and stapled by n
	Fuji_0.1.1_aarch64.dmg    signed, notarized and stapled
	log.txt                   every step's lines from the build log, and what each program printed between them
	app.notary.json           Apple's account of its examination of the app
	dmg.notary.json           and of the dmg
	verify.txt                Gatekeeper's verdicts, whose presence says the application is notarized
```

A folder is one release's whole record, and it stays on the Mac that publishes, because `applications/` is gitignored. The waiting room keeps nothing of its own: it reads where an application stands from the folder, Apple's history and the sidecars, every time. The copy of the app keeps a release on its way through Apple safe from `pnpm installer`, which signs the bundle with Fuji Desktop again and empties the folder the dmg was made in.

## Setting up a Mac

**A membership** in the Apple Developer Program, which issues the certificate and the key below.

**A Developer ID Application certificate in the login keychain.** Make it in the developer account under Certificates, Identifiers & Profiles, from a request Keychain Access writes, and its private key stays in the keychain that wrote the request. Export the two together as a password-protected .p12 and keep that off the Mac; a new Mac imports it. `security find-identity -v -p codesigning` should then list exactly one identity whose name begins Developer ID Application. The first signing stops at a dialog asking for the Mac's login password, and choosing Always Allow there lets every later build sign without asking.

Fuji names the identity by its kind, Developer ID Application, and codesign signs with the one identity whose name contains those words. The certificate's own name is its holder's legal name and team, so naming the kind keeps both out of the repository, and a renewed certificate needs no edit. Every signed copy still carries that name and team, where anyone can read them with `codesign -dvvv`. macOS keeps a user's permissions against a rule naming the team rather than one certificate, so a renewal costs no one a permission, and the move from Fuji Desktop costs each user one, once. While an old and a renewed certificate are both valid, the words match two identities, and b stops and says so; removing the older one from the login keychain settles it.

**An App Store Connect API key**, which n sends with and the waiting room reads Apple's history with. Make it in App Store Connect under Users and Access, then Integrations, with Developer access. Apple lets you download its .p8 once. Keep it in `~/.appstoreconnect/private_keys/`, where Apple's tools look, readable only by you, with `chmod 600`. A key lost or exposed is revoked on the same page and replaced, and nothing already shipped notices.

**Three values in `.env`** at the monorepo root, under the names Tauri reads:

```
APPLE_API_KEY=ABC123DEFG
APPLE_API_ISSUER=00000000-0000-0000-0000-000000000000
APPLE_API_KEY_PATH=/Users/you/.appstoreconnect/private_keys/AuthKey_ABC123DEFG.p8
```

The key ID and the issuer are identifiers, and the .p8 is the secret. u also needs the deploy values the same file holds for `pnpm upload`, and the essay in `scripts.js` describes those.

**uv**, for the dmg, which `pnpm installer` needs already: `brew install uv`. Then `pnpm install` at the root, and `pnpm waiting-room` here.

## Publishing a release

b, n, h and u, with the new application selected, and then a commit, because the sidecar in `desktop/release` is tracked. Or from desktop, `pnpm notarize`, `pnpm hash` and `pnpm upload`, the same three steps as the fallback's `pnpm installer`, `pnpm hash` and `pnpm upload`.

## When something goes wrong

- **Rejected.** `app.notary.json` or `dmg.notary.json` in the application's folder lists what Apple found. Fix it and b builds another application; the rejected one stays in the list as a record.
- **A build that failed.** `applications/building.txt` holds its output until the next b.
- **A check that failed after stapling.** The verdicts are in `verifying.txt` in the application's folder, and n tries the stapling and the checks again.
- **Anything else.** `log.txt` in the application's folder has what every step printed, and `build-log.txt` at the root has every step on this Mac, across every application and workspace.
