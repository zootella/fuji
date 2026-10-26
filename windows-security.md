# The sandbox process on Windows

What stays open on Windows for the sandbox process, and the Windows facts and sources the Security article draws on. Everything measured was measured on the Windows 10 box, Windows 10 Pro 22H2 build 19045, on 2026-10-25 and 2026-10-26.

## What stays open on Windows

- **Reads outside the folder and the network.** Both are allowed, and together they are the leak: a subverted sandbox process can read what the user can read and send it out. The job and the integrity level wall persistence, no write, no program, no registry, no autostart, and not confidentiality. The AppContainer is the answer to the network and waits on the folder question; a per-folder read wall has no Windows mechanism a process can put on itself.
- **Ending the main process.** The check opens the process that started it for `PROCESS_TERMINATE`, as the unix check sends signal 0 to its parent, and at Untrusted the open succeeds, while an open for its memory is refused; terminate sits in a process object's execute mapping, which the default label policy of no-read-up and no-write-up does not cover. A subverted sandbox process could end fuji, a denial and not an escape, where on the Mac and Linux it cannot signal the main process at all. The thing to try is the main process labeling its own process object no-execute-up at startup, after which the check's line says whether the open is refused.
- **Windows 11 by sight.** Everything here is Windows 10 22H2. The calls are all Windows 8 era or older, the job's nesting included, and Windows 11's WER, error mode and integrity mechanism are the same; the test box will say.
- **The handle-list race**, microseconds wide, documented in the code and real only if the main process spawned a `Command` on another thread during the start.
- **The thumbnailer's test of a header claiming more than the machine holds.** WIC declined its hundred-byte TIFF outright, "The component cannot be found", rather than reading the size and meeting the wall. The test now asks for a refusal from whichever layer refuses, and its 2 by 2 control, the same file small, says whether WIC reads that TIFF at all; the control has not run on Windows.

## Facts for the article

Stated the way a site page states them, without the rig or the trail, for the second half's Windows paragraphs.

- On Windows the sandbox process runs at the Untrusted integrity level inside a job object that allows it no other process, a gigabyte of memory, and none of the desktop: no clipboard, no window of another program's, no display settings. It cannot write a file or a registry value the user owns, start a program, or read or write another process's memory. It can read what the user can read, and it can reach the network.
- A crash of the sandbox process, a fault or an abort, ends that process alone. Fuji names the file that was being read and the code it ended with, and Windows shows no dialog and writes no report, because the sandbox process turns the system's reporter off for itself before it reads a byte.
- A file that stalls the sandbox process is ended by Stop at once, whatever it was doing, and fuji names the file it stalled on.
- If fuji itself ends while the sandbox process is running, the sandbox process ends with it within a few hundredths of a second.
- The sandbox process is started holding its three streams and nothing else of fuji's, with an environment of three names.
- These are the walls Chromium puts around a renderer on Windows, less the AppContainer that would close the network.

## Sources

- [SetErrorMode](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-seterrormode), the flag that keeps the reporter away, and [WER settings](https://learn.microsoft.com/en-us/windows/win32/wer/wer-settings), where `DefaultConsent` and its default of 1 are documented.
- [JOBOBJECT_BASIC_LIMIT_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information), the job's limits, and [Chromium's job.cc](https://chromium.googlesource.com/chromium/src/+/main/sandbox/win/src/job.cc), where the lockdown level's UI restrictions are set.
- [__fastfail](https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail?view=msvc-170), why an abort skips the exception path, and [Electron's fast-fail reporting helper](https://github.com/electron/electron/pull/53851), which met the same gap.
- [WerSetFlags](https://learn.microsoft.com/en-us/windows/win32/api/werapi/nf-werapi-wersetflags), which does not stop a report.
- [Designing applications to run at a low integrity level](https://learn.microsoft.com/en-us/previous-versions/dotnet/articles/bb625960(v=msdn.10)), Microsoft's account of what Low refuses, and [Chromium's sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md), the token, job, desktop and AppContainer together.
- [rust-lang/rust#73281](https://github.com/rust-lang/rust/issues/73281) and [libs-team#264](https://github.com/rust-lang/libs-team/issues/264), the standard library's handle inheritance, and [std::process::Command is a bad citizen on Windows](https://purplesyringa.moe/blog/std-process-command-is-a-bad-citizen-on-windows/), which lays it out.
- [The hidden danger of forgetting to specify SystemRoot in a custom environment block](https://jpassing.com/2009/12/28/the-hidden-danger-of-forgetting-to-specify-systemroot-in-a-custom-environment-block), why the environment is three names and not none.
