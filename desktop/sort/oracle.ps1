# Windows' own answers, for sort/test.js to hold WindowsSort.js against: this script runs names through the functions Explorer itself uses and writes what they said. It reads a file of names, each as a count of UTF-16 code units in two bytes and then the units, low byte first, so a name can hold anything a file's name can and more. With -Mode keys it writes each name's sort key from LCMapStringEx, under the two flags StrCmpLogicalW compares with, as a count in two bytes and then the key's bytes. With -Mode compare it takes the names two at a time and writes a byte a pair from StrCmpLogicalW: 0 for the first before the second, 1 for the same, 2 for after. With -Mode shell and -Folder it does the same with the names as files in that folder, asked of the shell's own folder object through IShellFolder::CompareIDs, which is what Explorer's view sorts by and one step nearer to it than the function. With -Mode explorer and -Folder it opens a real Explorer window on each folder inside that one and writes the order the window shows. With -Mode version it writes nothing and says which sorting table this Windows has.
param([string]$Mode = 'version', [string]$In, [string]$Out, [string]$Folder)
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Collections.Generic;
using System.Runtime.InteropServices;
[ComImport, Guid("000214E6-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IShellFolder {// the first five methods, in the order the interface declares them, since a call finds its method by its place
	[PreserveSig] int ParseDisplayName(IntPtr window, IntPtr context, [MarshalAs(UnmanagedType.LPWStr)] string name, IntPtr eaten, out IntPtr item, IntPtr attributes);
	[PreserveSig] int EnumObjects(IntPtr window, int flags, out IntPtr list);
	[PreserveSig] int BindToObject(IntPtr item, IntPtr context, ref Guid id, [MarshalAs(UnmanagedType.Interface)] out object made);
	[PreserveSig] int BindToStorage(IntPtr item, IntPtr context, ref Guid id, out IntPtr made);
	[PreserveSig] int CompareIDs(IntPtr column, IntPtr first, IntPtr second);
}
public static class Oracle {
	[DllImport("shell32.dll", CharSet = CharSet.Unicode)]
	static extern int SHParseDisplayName(string name, IntPtr context, out IntPtr item, uint asked, out uint answered);
	[DllImport("shell32.dll")]
	static extern int SHBindToObject(IntPtr parent, IntPtr item, IntPtr context, ref Guid id, [MarshalAs(UnmanagedType.Interface)] out object made);
	public static string Shell(string folder, string from, string to) {
		IntPtr item; uint answered; object made;
		Guid id = new Guid("000214E6-0000-0000-C000-000000000046");
		if (SHParseDisplayName(folder, IntPtr.Zero, out item, 0, out answered) != 0 || SHBindToObject(IntPtr.Zero, item, IntPtr.Zero, ref id, out made) != 0) throw new Exception("could not open the folder in the shell");
		var shell = (IShellFolder)made;
		var names = Names(from);
		var items = new Dictionary<string, IntPtr>();
		foreach (string name in names) if (!items.ContainsKey(name)) { IntPtr child; if (shell.ParseDisplayName(IntPtr.Zero, IntPtr.Zero, name, IntPtr.Zero, out child, IntPtr.Zero) != 0) throw new Exception("the shell could not find a file in the folder"); items[name] = child; }// never freed, in a process that ends when this does
		using (var w = new BinaryWriter(File.Create(to))) for (int i = 0; i + 1 < names.Count; i += 2) {
			int r = (short)(shell.CompareIDs(IntPtr.Zero, items[names[i]], items[names[i + 1]]) & 0xffff);// column 0 is the name, and the answer is the low half of the result
			w.Write((byte)(r < 0 ? 0 : r > 0 ? 2 : 1));
		}
		return (names.Count / 2) + " pairs";
	}
	[DllImport("shlwapi.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
	static extern int StrCmpLogicalW(string a, string b);
	[DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
	static extern int LCMapStringEx(IntPtr locale, uint flags, string a, int na, byte[] key, int nkey, IntPtr v, IntPtr r, IntPtr l);
	[StructLayout(LayoutKind.Sequential)]
	struct NlsVersionInfoEx { public uint size, nls, defined, effective; public Guid sort; }
	[DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
	static extern bool GetNLSVersionEx(int function, IntPtr locale, ref NlsVersionInfoEx info);
	static List<string> Names(string path) {
		byte[] b = File.ReadAllBytes(path);
		var names = new List<string>();
		for (int i = 0; i < b.Length;) {
			int n = b[i] | (b[i + 1] << 8); i += 2;
			char[] c = new char[n];
			for (int k = 0; k < n; k++, i += 2) c[k] = (char)(b[i] | (b[i + 1] << 8));
			names.Add(new string(c));
		}
		return names;
	}
	public static string Keys(string from, string to) {
		var names = Names(from);
		byte[] key = new byte[65536];
		using (var w = new BinaryWriter(File.Create(to))) foreach (string name in names) {
			int n = name.Length == 0 ? 0 : LCMapStringEx(IntPtr.Zero, 0x400 | 0x1 | 0x8, name, name.Length, key, key.Length, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);// LCMAP_SORTKEY | NORM_IGNORECASE | SORT_DIGITSASNUMBERS, in the user's locale, which a null name asks for
			if (n > 0) n--;// without the 0 that ends the key
			w.Write((ushort)n); w.Write(key, 0, n);
		}
		return names.Count + " keys";
	}
	public static string Compare(string from, string to) {
		var names = Names(from);
		using (var w = new BinaryWriter(File.Create(to))) for (int i = 0; i + 1 < names.Count; i += 2) {
			int r = StrCmpLogicalW(names[i], names[i + 1]);
			w.Write((byte)(r < 0 ? 0 : r > 0 ? 2 : 1));
		}
		return (names.Count / 2) + " pairs";
	}
	public static string Version() {
		var info = new NlsVersionInfoEx(); info.size = (uint)Marshal.SizeOf(typeof(NlsVersionInfoEx));
		if (!GetNLSVersionEx(1, IntPtr.Zero, ref info)) return "GetNLSVersionEx failed";// COMPARE_STRING, in the user's locale
		return "NLS version 0x" + info.nls.ToString("x8") + ", sort " + info.sort;
	}
}
'@
if ($Mode -eq 'keys') { [Oracle]::Keys($In, $Out) }
elseif ($Mode -eq 'compare') { [Oracle]::Compare($In, $Out) }
elseif ($Mode -eq 'shell') { [Oracle]::Shell($Folder, $In, $Out) }
elseif ($Mode -eq 'explorer') {# a real Explorer window on each folder inside -Folder, sorted by name, and the order it shows, written to -Out as UTF-16 lines of a folder's name, a tab, and a file's name, top to bottom; each window opens, is read, and closes
	$shell = New-Object -ComObject Shell.Application
	$lines = New-Object System.Collections.Generic.List[string]
	foreach ($inside in [IO.Directory]::GetDirectories($Folder)) {
		$before = @($shell.Windows() | ForEach-Object { $_.HWND })
		$shell.Open($inside)
		$window = $null
		foreach ($try in 1..100) {# the window is another process's, and arrives when it arrives
			Start-Sleep -Milliseconds 100
			$window = $shell.Windows() | Where-Object { $before -notcontains $_.HWND -and $_.Document -and $_.Document.Folder.Self.Path -eq $inside } | Select-Object -First 1
			if ($window) { break }
		}
		if (-not $window) { throw "no Explorer window opened on $inside" }
		$window.Document.SortColumns = 'prop:System.ItemNameDisplay;'# by name, ascending, whatever the folder was sorted by before
		Start-Sleep -Milliseconds 300
		foreach ($item in $window.Document.Folder.Items()) { $lines.Add([IO.Path]::GetFileName($inside) + "`t" + [IO.Path]::GetFileName($item.Path)) }# the items come in the view's order, which sorting the other way was seen to turn over; the path's last part rather than the name shown, which drops an extension Explorer is hiding
		$window.Quit()
	}
	[IO.File]::WriteAllLines($Out, $lines, [Text.Encoding]::Unicode)
	"$($lines.Count) files"
}
else { "$([Oracle]::Version()), $([System.Environment]::OSVersion.VersionString), user locale $((Get-Culture).Name)" }
