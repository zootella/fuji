# Download Fuji

One package per system, and the architecture matters as much as the name: a build for ARM will not run on an x86 machine, and the reverse. Take the one that matches your computer, and if you are not sure which that is, the notes under the list say who each file is for.

<Download file="fuji.dmg"            platform="macOS"   system="Apple silicon" />
<Download file="fuji.exe"            platform="Windows" system="64-bit Intel and AMD" />
<Download file="fuji.arm64.deb"      platform="Linux"   system="Raspberry Pi, and other ARM machines running Debian or Ubuntu" />
<Download file="fuji.amd64.deb"      platform="Linux"   system="Debian and Ubuntu on 64-bit Intel and AMD" />
<Download file="fuji.x86_64.rpm"     platform="Linux"   system="Fedora and RHEL on 64-bit Intel and AMD" />
<Download file="fuji.x86_64.flatpak" platform="Linux"   system="any distribution, sandboxed, 64-bit Intel and AMD" />

Arch Linux and its relatives have no file of their own here. The Flatpak runs on all of them. See below.

## Which file, and for what

**macOS.** `fuji.dmg` is built for Apple silicon — the M-series machines. There is no Intel build. Open the disk image and drag Fuji into your Applications folder.

**Windows.** `fuji.exe` is the *installer*, for 64-bit Intel and AMD processors, and running it walks through a short wizard. It is not the application itself, which the installer unpacks and puts where Windows expects it.

**Debian, Ubuntu and their relatives.** There are two, and the difference is the processor, which each filename states. `fuji.amd64.deb` is for a 64-bit Intel or AMD desktop, which is almost certainly what you have. `fuji.arm64.deb` is the **Raspberry Pi** one, and suits other ARM machines too; it will not run on an ordinary desktop. If you pick wrong, your package manager refuses the file rather than installing something that cannot run. Both install with your package manager or a double-click, and both work on Debian 12 and later, Ubuntu 24.04 LTS and later, and Linux Mint 22 and later.

**Fedora, RHEL, Rocky, AlmaLinux.** `fuji.x86_64.rpm` is the package for those and their relatives, for 64-bit Intel and AMD.

**Any distribution at all, on a 64-bit Intel or AMD machine.** `fuji.x86_64.flatpak` is a Flatpak bundle, which carries its own libraries and runs in a sandbox, so it does not care what your distribution ships — though it does still care what processor you have, and this one is not for a Raspberry Pi. Download it and install it with `flatpak install ./fuji.x86_64.flatpak`. It is the only one of these that works on an immutable system like SteamOS on the Steam Deck, or Bazzite, where the root filesystem is read-only and an ordinary package cannot be installed at all.

Fuji reads the folders you point it at, so the Flatpak asks for access to your filesystem. Your software centre will say so when you install it, and it is the same access the other packages have without asking.

**Arch, Manjaro, EndeavourOS, CachyOS.** There is no AUR package. The Flatpak above runs on every one of these. If your system did not come with `flatpak`, it is in Arch's own repositories, and then `flatpak install ./fuji.x86_64.flatpak` is the whole of it.

## The downloads are not code-signed

Fuji is not signed by Apple or by a Windows certificate authority, and until it is, each system will say so in its own way.

On Windows, a browser attaches a mark-of-the-web to anything it downloads, and that mark is what raises the SmartScreen warning when you run the installer. The screen has a **More info** link, and the button to continue sits behind it. The warning is about where the file came from, not about the file itself: a copy you built on your own machine carries no mark and runs without it.

On macOS, an application downloaded from the web is quarantined, and the first attempt to open it is refused because Fuji carries no Apple signature. Control-click Fuji in your Applications folder, choose Open, and the dialog that appears has a button that proceeds. You do this once.

## Checking the hash

Every package is published with its SHA-256 beside it, and comparing the two catches a download that was corrupted or truncated on the way to you, along with the ordinary mistake of having grabbed the wrong file.

```bash
shasum -a 256 fuji.dmg        # macOS and Linux
```

```powershell
Get-FileHash fuji.exe         # Windows PowerShell
```

Compare the result against the hash on this page. If they match, you have the whole file, exactly as it was published.

## Building it yourself

Fuji is open source, and the [repository](https://github.com/zootella/fuji) has what each platform needs to build it — the toolchains, the commands, and where the output lands. Building from source is also the only way to get a copy for an architecture no release covers.
