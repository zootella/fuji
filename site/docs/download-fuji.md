# Download Fuji

There is one package for each system, and the processor architecture matters as well: a build for ARM does not run on an x86 machine, and an x86 build does not run on ARM. Choose the file that matches your computer.

<DownloadLink file="fuji.dmg"            platform="macOS"   system="Apple silicon" />
<DownloadLink file="fuji.exe"            platform="Windows" system="64-bit Intel and AMD" />
<DownloadLink file="fuji.arm64.deb"      platform="Linux"   system="Raspberry Pi, and other ARM machines running Debian or Ubuntu" />
<DownloadLink file="fuji.amd64.deb"      platform="Linux"   system="Debian and Ubuntu on 64-bit Intel and AMD" />
<DownloadLink file="fuji.x86_64.rpm"     platform="Linux"   system="Fedora and RHEL on 64-bit Intel and AMD" />
<DownloadLink file="fuji.x86_64.flatpak" platform="Linux"   system="any distribution, sandboxed, 64-bit Intel and AMD" />

Arch Linux and its relatives have no package of their own. The Flatpak runs on all of them.

## Getting past Windows and macOS warnings

Both systems ask you to confirm the first run of a program downloaded through a browser. A browser marks every file it downloads, and that mark is what triggers the question. A copy you fetch with the command line below, or build from source, carries no mark and opens without one.

### macOS: installing and running Fuji

1. Download `fuji.dmg`, open it, and drag Fuji into the Applications folder.
2. Double-click Fuji in your Applications folder. macOS asks whether you want to open an app downloaded from the Internet. Click **Open**.
3. Fuji opens, and later launches ask nothing.

### Windows: installing and running Fuji

1. Download `fuji.exe` and run it.
2. [Microsoft Defender SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/) displays a blue screen with the message **Windows protected your PC. Microsoft Defender SmartScreen prevented an unrecognized app from starting.** Click **More info**, then click **Run anyway**.
3. Nothing else appears: the installer puts Fuji in place, adds it to the Start menu, and starts it, in about a second. Running a newer installer later updates Fuji the same way, and keeps the file types you chose to open with it.

If a Windows 11 computer instead displays a message from [Smart App Control](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions), with no **Run anyway** button, that feature has to be turned off before the installer can run. It is in the Windows Security app under **App & browser control**, and on earlier builds of Windows 11, turning it off lasts until Windows is reinstalled.

## Alternatively: Get Fuji using the command line

As an alternative to downloading through your browser and then encountering all these hurdles, you can get Fuji using your command line. This method has the added benefit of automatically checking the hash! These steps work without needing an administrator account.

**macOS:** Click search in the upper right, and type **Terminal**. Copy and paste the command below, and hit the **Return** key. Two spaces follow the hash value; this is necessary for the command to work.

<DownloadCommand file="fuji.dmg">

```bash
cd ~/Downloads && \
curl -fsSL -o fuji_setup.dmg https://fujidesktop.app/fuji.dmg && \
echo "0000000000000000000000000000000000000000000000000000000000000000  fuji_setup.dmg" | shasum -a 256 -c && \
open fuji_setup.dmg
```

</DownloadCommand>

**Windows:** Click the Start menu and type **PowerShell**. Copy and paste the command below, and hit the **Enter** key.

<DownloadCommand file="fuji.exe">

```powershell
cd ~\Downloads
curl.exe -fsSL -o fuji_setup.exe https://fujidesktop.app/fuji.exe
if ((Get-FileHash fuji_setup.exe).Hash -eq '0000000000000000000000000000000000000000000000000000000000000000') { .\fuji_setup.exe } else { 'The hash does not match. Fuji was not installed.' }
```

</DownloadCommand>

The macOS and Windows commands do the same three things. They save `fuji_setup.dmg` or `fuji_setup.exe` in your _Downloads_ folder. They compute the file's SHA-256 hash and make sure it is correct. Lastly, they open the file, same as a double-click. On macOS the disk image appears, and you drag Fuji into _Applications_. On Windows the installer puts Fuji in place and starts it. Neither system shows its warning, because that warning is triggered by a mark browsers add to files they download, and a file fetched by <code>curl</code> carries none.

## Even better: Build Fuji yourself

Fuji is open source, and the [repository](https://github.com/zootella/fuji) has what each platform needs to build it — the toolchains, the commands, and where the output lands. Building from source is also the only way to get a copy for an architecture no release covers.

Open source also means Fuji can go further than its settings do. Clone the repository, have a coding agent add the feature you want or change how Fuji behaves, and build a copy that is yours.

## Which file, and for what

**macOS.** `fuji.dmg` is built for Apple silicon — the M-series machines. There is no Intel build. Open the disk image and drag Fuji into your Applications folder.

**Windows.** `fuji.exe` is the *installer*, for 64-bit Intel and AMD processors, and running it installs Fuji with no questions and then starts it. It is not the application itself, which the installer unpacks and puts where Windows expects it.

**Debian, Ubuntu and their relatives.** There are two, and the difference is the processor, which each filename states. `fuji.amd64.deb` is for a 64-bit Intel or AMD desktop, which is almost certainly what you have. `fuji.arm64.deb` is the **Raspberry Pi** one, and suits other ARM machines too; it will not run on an ordinary desktop. If you pick wrong, your package manager refuses the file rather than installing something that cannot run. Both install with your package manager or a double-click, and both work on Debian 12 and later, Ubuntu 24.04 LTS and later, and Linux Mint 22 and later.

**Fedora, RHEL, Rocky, AlmaLinux.** `fuji.x86_64.rpm` is the package for those and their relatives, for 64-bit Intel and AMD.

**Any distribution at all, on a 64-bit Intel or AMD machine.** `fuji.x86_64.flatpak` is a Flatpak bundle, which carries its own libraries and runs in a sandbox, so it does not care what your distribution ships — though it does still care what processor you have, and this one is not for a Raspberry Pi. Download it and install it with `flatpak install ./fuji.x86_64.flatpak`. It is the only one of these that works on an immutable system like SteamOS on the Steam Deck, or Bazzite, where the root filesystem is read-only and an ordinary package cannot be installed at all.

Fuji reads the folders you point it at, so the Flatpak asks for access to your filesystem. Your software center will say so when you install it, and it is the same access the other packages have without asking.

**Arch, Manjaro, EndeavourOS, CachyOS.** There is no AUR package. The Flatpak runs on every one of these. If your system did not come with `flatpak`, it is in Arch's own repositories, and then `flatpak install ./fuji.x86_64.flatpak` installs Fuji.

## Checking the hash

Every package is published with its SHA-256 beside it, and comparing the two catches a download that was corrupted or truncated on the way to you, along with the ordinary mistake of having grabbed the wrong file.

```bash
shasum -a 256 fuji.dmg        # macOS, in Terminal
```

```powershell
Get-FileHash fuji.exe         # Windows, in PowerShell
```

```bash
sha256sum fuji.amd64.deb      # Linux
```

Compare the result against the hash on this page. If they match, you have the whole file, exactly as it was published.
