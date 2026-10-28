# Always Offline

Fuji never goes online. Not a box you can uncheck, and not a feature we forgot: no code in Fuji opens a connection. It works exactly the same online and offline, and there's nothing to configure, check or worry about.

## Every app was offline

For the first twenty years, every app opened files and nothing sent or received, because there was nothing to connect to. The Apple II, the Commodore PET and the TRS-80 [all came out in 1977](https://www.computerhistory.org/timeline/1977/) and saved their programs to cassette tape, and the IBM PC followed [in August 1981](https://www.ibm.com/history/personal-computer) with 16 kilobytes of memory and no disk drive. Going online was something a person did: they bought [a Smartmodem](https://en.wikipedia.org/wiki/Hayes_Microcomputer_Products) for $299 that moved 300 bits a second, dialed [a bulletin board](https://ethw.org/Bulletin_Board_Systems) like the one Ward Christensen and Randy Suess put on a Chicago phone line in February 1978, got what they came for, and hung up.

Fast forward twenty-five years, and a connection is something every device just has, phone, laptop or Chromebook, unless you flip on airplane mode. A new kind of app showed up that exists to send and receive: a browser, an email app, Instagram. Nobody expects them to work offline. But a photo editor, a video player and a file manager open the files already on your disk, and a connection adds nothing to that job. When one of them stops working offline anyway, somebody made that choice, and it wasn't you.

## Somebody else's reasons

When an app whose job is your files connects anyway, it's for somebody else's reasons:

- **To confirm who you are.** A sign-in before the first file opens.
- **To check that you paid.** A subscription asks a server, on its own schedule, whether you are still a customer.
- **To sell you more.** You bought one app, so here is the whole suite at one low price, this week only, and a helper that starts with Windows and sits in the tray to make sure you hear about it.
- **To ask about your files.** A metadata server trades what you just opened for cover art and a track name, one file at a time.
- **To report on you and your activities.** Usage statistics, crash reports, and legal compliance.

The first three are a nuisance. The last two are about you, and that's where a nuisance turns into a privacy problem. VLC, the free player, is a fair example, because it asks first. [Its first-run dialog](https://github.com/videolan/vlc/blob/3.0.x/modules/gui/qt/dialogs/firstrun.cpp), titled *Privacy and Network Access Policy*, says that fetching cover art and track names "may entail identifying some of your media files to third party entities," and [VideoLAN's privacy page](https://videolan.org/privacy.html) names them: last.fm, Google Images and a MusicBrainz server of VideoLAN's own. When a patch in 2016 proposed that turning the setting off should also stop the player fetching album art, one of its developers answered on the project's mailing list:

> If you want to prevent VLC from using the network, then firewall it. I cannot see any sense for the sake of privacy in blocking the art fetcher, but letting other forms of automatic network access.
>
> Rémi Denis-Courmont, VLC developer, [on the vlc-devel list](https://mailman.videolan.org/pipermail/vlc-devel/2016-December/110884.html), December 11, 2016.

## Always offline

Fuji opens files, and that's its whole job. It has no account, so nothing asks who you are. It costs nothing, so nothing checks whether you paid. It sells nothing, so there's nothing more to sell you. It asks no server about your files, because everything it needs to know about a file is... in the file! And it has no crash reporter, no usage statistics and no address to send them to. There's no first-run dialog, because there's nothing to consent to, and no firewall rule to write, because there's nothing to block.

Anyone can check. [Fuji's configuration](https://github.com/zootella/fuji/blob/main/desktop/src-tauri/tauri.conf.json) lets the page connect to nothing but its own Rust, and [the recipe](https://github.com/zootella/fuji/blob/main/desktop/libmpv/build.py) that builds its media library leaves out FFmpeg's network code and every way of reading except from a file on disk. Fuji is always offline.

## Aligned thinking: Local-first software

Reading up on the history of apps and connections for this page turned up a group that has thought carefully about the balance between the two, and about what a person is owed by their software: Ink & Switch, "an independent research lab exploring the future of tools for thought." Their 2019 essay [Local-first software: You own your data, in spite of the cloud](https://www.inkandswitch.com/essay/local-first/), by Martin Kleppmann, Adam Wiggins, Peter van Hardenberg and Mark McGranaghan, starts from the observation that "the cloud gives us collaboration, but old-fashioned apps give us ownership," and sets out to have both, under seven ideals:

> 1\. No spinners: your work at your fingertips\
> 2\. Your work is not trapped on one device\
> 3\. The network is optional\
> 4\. Seamless collaboration with your colleagues\
> 5\. The Long Now\
> 6\. Security and privacy by default\
> 7\. You retain ultimate ownership and control
>
> The seven ideals of [Local-first software](https://www.inkandswitch.com/essay/local-first/), Ink & Switch, 2019.

The fifth shares its name with [the foundation](https://longnow.org/) building a clock to keep time for the next ten thousand years, and means that "your work should continue to be accessible indefinitely, even after the company that produced the software is gone." The lab builds all seven on a family of algorithms called *conflict-free replicated data types*, in its own open-source library, [Automerge](https://automerge.org/).

Fuji isn't local-first software, because Fuji is local only: there is nothing to sync and nobody to sync with. But the sympathies run the same way, and it's a good read.
