# Download Fuji

One installer per system, and the architecture matters as much as the name: a build for ARM will not run on an x86 machine. Take the one that matches your computer, and if you are not sure which that is, the notes under the list say who each file is for.

<Download file="fuji.dmg" platform="macOS"   system="Apple silicon" />
<Download file="fuji.exe" platform="Windows" system="64-bit Intel and AMD" />
<Download file="fuji.deb" platform="Linux"   system="Debian and Ubuntu on ARM" />

## Which file, and for what

**macOS.** `fuji.dmg` is built for Apple silicon — the M-series machines. There is no Intel build. Open the disk image and drag Fuji into your Applications folder.

**Windows.** `fuji.exe` is the *installer*, for 64-bit Intel and AMD processors, and running it walks through a short wizard. It is not the application itself, which the installer unpacks and puts where Windows expects it.

**Linux.** `fuji.deb` is a Debian package — Debian, Ubuntu, and the distributions built on them — and it is not published yet. When it arrives it will be an ARM build rather than an x86 one, so it will suit a Raspberry Pi and machines like it, and it will not run on an ordinary x86 desktop.

## The downloads are not code-signed

Fuji is not signed by Apple or by a Windows certificate authority, and until it is, each system will say so in its own way.

On Windows, a browser attaches a mark-of-the-web to anything it downloads, and that mark is what raises the SmartScreen warning when you run the installer. The screen has a **More info** link, and the button to continue sits behind it. The warning is about where the file came from, not about the file itself: a copy you built on your own machine carries no mark and runs without it.

On macOS, an application downloaded from the web is quarantined, and the first attempt to open it is refused because Fuji carries no Apple signature. Control-click Fuji in your Applications folder, choose Open, and the dialog that appears has a button that proceeds. You do this once.

## Checking the hash

Every installer is published with its SHA-256 beside it, and comparing the two catches a download that was corrupted or truncated on the way to you, along with the ordinary mistake of having grabbed the wrong file.

```bash
shasum -a 256 fuji.dmg        # macOS and Linux
```

```powershell
Get-FileHash fuji.exe         # Windows PowerShell
```

Compare the result against the hash on this page. If they match, you have the whole file, exactly as it was published.

## Building it yourself

Fuji is open source, and the [repository](https://github.com/zootella/fuji) has what each platform needs to build it — the toolchains, the commands, and where the output lands. Building from source is also the only way to get a copy for an architecture no release covers.
