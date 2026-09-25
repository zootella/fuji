# Download Fuji

There is one package for each system, and the processor architecture matters as well: a build for ARM does not run on an x86 machine, and an x86 build does not run on ARM. Choose the file that matches your computer.

<DownloadLink file="fuji.dmg"            platform="macOS"   system="Apple silicon" />
<DownloadLink file="fuji.exe"            platform="Windows" system="64-bit Intel and AMD" />
<DownloadLink file="fuji.arm64.deb"      platform="Linux"   system="Raspberry Pi, and other ARM machines running Debian or Ubuntu" />
<DownloadLink file="fuji.amd64.deb"      platform="Linux"   system="Debian and Ubuntu on 64-bit Intel and AMD" />
<DownloadLink file="fuji.x86_64.rpm"     platform="Linux"   system="Fedora and RHEL on 64-bit Intel and AMD" />
<DownloadLink file="fuji.x86_64.flatpak" platform="Linux"   system="any distribution, sandboxed, 64-bit Intel and AMD" />

Arch Linux and its relatives have no package of their own. The Flatpak runs on all of them.

## Which file, and for what

**macOS.** `fuji.dmg` is built for Apple silicon — the M-series machines. There is no Intel build. Open the disk image and drag Fuji into your Applications folder.

**Windows.** `fuji.exe` is the *installer*, for 64-bit Intel and AMD processors, and running it walks through a short wizard. It is not the application itself, which the installer unpacks and puts where Windows expects it.

**Debian, Ubuntu and their relatives.** There are two, and the difference is the processor, which each filename states. `fuji.amd64.deb` is for a 64-bit Intel or AMD desktop, which is almost certainly what you have. `fuji.arm64.deb` is the **Raspberry Pi** one, and suits other ARM machines too; it will not run on an ordinary desktop. If you pick wrong, your package manager refuses the file rather than installing something that cannot run. Both install with your package manager or a double-click, and both work on Debian 12 and later, Ubuntu 24.04 LTS and later, and Linux Mint 22 and later.

**Fedora, RHEL, Rocky, AlmaLinux.** `fuji.x86_64.rpm` is the package for those and their relatives, for 64-bit Intel and AMD.

**Any distribution at all, on a 64-bit Intel or AMD machine.** `fuji.x86_64.flatpak` is a Flatpak bundle, which carries its own libraries and runs in a sandbox, so it does not care what your distribution ships — though it does still care what processor you have, and this one is not for a Raspberry Pi. Download it and install it with `flatpak install ./fuji.x86_64.flatpak`. It is the only one of these that works on an immutable system like SteamOS on the Steam Deck, or Bazzite, where the root filesystem is read-only and an ordinary package cannot be installed at all.

Fuji reads the folders you point it at, so the Flatpak asks for access to your filesystem. Your software center will say so when you install it, and it is the same access the other packages have without asking.

**Arch, Manjaro, EndeavourOS, CachyOS.** There is no AUR package. The Flatpak runs on every one of these. If your system did not come with `flatpak`, it is in Arch's own repositories, and then `flatpak install ./fuji.x86_64.flatpak` installs Fuji.

## Running Fuji for the first time

Fuji is deliberately not signed with an Apple Developer ID or a Windows code-signing certificate. As a result, each operating system stops the first run of a downloaded file and asks you to confirm it. The warning concerns where the file came from rather than the file itself: a browser marks every file it downloads, and that mark is what triggers the check. A copy you build from source carries no mark and opens without any of this.

The instructions are in four parts. For each system, the first part gives the steps to install and run Fuji with the system's default settings, which you follow once per computer. The second part, separately, describes the settings that turn this kind of check off for every program.

### macOS: installing and running Fuji

1. Download `fuji.dmg`, open it, and drag Fuji into the Applications folder.
2. Double-click Fuji in your Applications folder. macOS displays the message **"Fuji" Not Opened. Apple could not verify "Fuji" is free of malware that may harm your Mac or compromise your privacy**, with the buttons **Done** and **Move to Trash**. Click **Done**.
3. Open System Settings, select **Privacy & Security**, and scroll down to the **Security** section. A message states that Fuji was blocked to protect your Mac, with an **Open Anyway** button beside it. Click the button and enter your login password when asked.
4. Fuji opens, and macOS saves the exception, so later launches need no confirmation.

The check is performed by [Gatekeeper](https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web), and Apple documents these steps in [Open a Mac app from an unknown developer](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unknown-developer-mh40616/mac). The **Open Anyway** button remains available for about an hour after the refusal; if it has disappeared, double-click Fuji again to bring it back. Control-clicking Fuji and choosing Open, the older method of bypassing this dialog, no longer works on macOS Sequoia or later, as Apple's page [Safely open apps on your Mac](https://support.apple.com/en-us/102445) describes.

### macOS: turning the check off for every program

The setting is [Allow applications from](https://support.apple.com/guide/mac-help/change-privacy-security-settings-on-mac-mchl211c911f/mac), in System Settings under **Privacy & Security**, in the **Security** section. It offers two choices, **App Store** and **App Store & Known Developers**. A third choice, **Anywhere**, stops the check for every program. macOS hides it until you run one command in Terminal, after which it appears in the menu and can be chosen:

```bash
sudo spctl --global-disable   # Sequoia and later; --master-disable on earlier versions
```

### Windows: installing and running Fuji

1. Download `fuji.exe` and run it.
2. [Microsoft Defender SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/) displays a blue screen with the message **Windows protected your PC. Microsoft Defender SmartScreen prevented an unrecognized app from starting.** Click **More info**, then click **Run anyway**.
3. The installer's wizard follows. Once Fuji is installed, it runs without further prompts.

If a Windows 11 computer instead displays a message from Smart App Control, with no **Run anyway** button, that feature has to be turned off before the installer can run.

### Windows: turning the check off for every program

Two settings are involved, and both are in the [Windows Security](https://support.microsoft.com/en-us/windows/security/windows-security/windows-security-app-overview) app under [App & browser control](https://support.microsoft.com/en-us/windows/security/windows-security/app-browser-control-in-the-windows-security-app).

The first is SmartScreen itself. Under **Reputation-based protection settings**, turn off **Check apps and files**. Changing it requires an administrator account, and it stops the check for every download, so Windows no longer checks downloaded programs before they run.

The second is [Smart App Control](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions), which exists on Windows 11 only. When it is turned on, it blocks any unsigned program outright, and Microsoft's documentation states that there is no way to allow an individual program. Under **Smart App Control settings**, choose **Off**. Recent Windows updates allow it to be turned back on afterwards. On earlier builds, turning it off is permanent until Windows is reinstalled, so check your update level before you change it.

## Use the command line

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

The macOS and Windows commands do the same three things. They save `fuji_setup.dmg` or `fuji_setup.exe` in your _Downloads_ folder. They compute the file's SHA-256 hash and make sure it is correct. Lastly, they open the file, same as a double-click. On macOS the disk image appears, and you drag Fuji into _Applications_. On Windows the setup wizard begins. Neither system shows its warning, because that warning is triggered by a mark browsers add to files they download, and a file fetched by <code>curl</code> carries none.

## Why Fuji is not signed

Fuji is a multimedia file manager designed with privacy and precision in mind. The project is dedicated to open source and the open web, and takes security seriously. Both commitments lead to the same principle: the person who owns a computer should control what runs on it, and Fuji is built so that you do.

The industry has been moving in the other direction for years. More of what runs on your computer must first be approved somewhere else: registered with a [developer program](https://developer.apple.com/programs/), [signed with a certificate](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options) the vendor issues and can revoke, [submitted for review](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), and checked against a server at launch. Each step is presented as a security measure, and each one moves a decision from your machine to a company's. We disagree with that direction, and we opt out of it. Fuji is not registered with an app store or a developer program, and it does not enter their signing and notarization processes. This is a deliberate decision, and it is the reason each system stops the first run.

Fuji relies instead on an older method that anyone can verify. The source code is public, so anyone can read what the program does, and build the copy they run from that code themselves. Each installer is built on a machine we control and published beside its SHA-256 hash. The hash is also committed to the repository, so the record of what was published is public and dated. Checking the hash of a downloaded file against it takes one command. A matching hash proves that the file is exactly what we built, byte for byte, which is the assurance a signature provides. It proves it on your own machine, without anyone else's involvement. We believe this is more secure, not less, because it rests on a public record rather than a private key, and on your own computer rather than a vendor's servers. **More importantly, it places you in control.** Your computer is your property, and the choice of what runs on it belongs to you.

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

## Building it yourself

Fuji is open source, and the [repository](https://github.com/zootella/fuji) has what each platform needs to build it — the toolchains, the commands, and where the output lands. Building from source is also the only way to get a copy for an architecture no release covers.
