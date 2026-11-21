// the Mac's own answers, for sort/test.js to hold MacSort.js against: this script runs names through localizedStandardCompare, the method Apple says orders names as Finder does, and writes what it said. swift oracle.swift compare in out reads a file of names, each as a count of UTF-16 code units in two bytes and then the units, low byte first, takes them two at a time, and writes a byte a pair: 0 for the first before the second, 1 for the same, 2 for after. With no arguments it says which macOS this is and which language the comparison follows, since the order is the user's language's own
import Foundation

func names(_ path: String) -> [NSString] {
	let bytes = [UInt8](try! Data(contentsOf: URL(fileURLWithPath: path)))
	var found: [NSString] = []
	var i = 0
	while i + 1 < bytes.count {
		let count = Int(bytes[i]) | Int(bytes[i + 1]) << 8
		i += 2
		var units = [unichar](repeating: 0, count: count)
		for k in 0..<count { units[k] = unichar(bytes[i]) | unichar(bytes[i + 1]) << 8; i += 2 }
		found.append(NSString(characters: units, length: count))
	}
	return found
}

let arguments = CommandLine.arguments
if arguments.count >= 4 && arguments[1] == "compare" {
	let all = names(arguments[2])
	var said = [UInt8]()
	var i = 0
	while i + 1 < all.count {
		let order = all[i].localizedStandardCompare(all[i + 1] as String)
		said.append(order == .orderedAscending ? 0 : order == .orderedSame ? 1 : 2)
		i += 2
	}
	try! Data(said).write(to: URL(fileURLWithPath: arguments[3]))
	print("\(all.count / 2) pairs")
} else if arguments.count >= 4 && arguments[1] == "finder" {// a real Finder window on each folder inside the one given, in list view sorted by name, and the order it shows, written as UTF-16 lines of a folder's name, a tab, and a file's name, top to bottom, as the explorer mode of oracle.ps1 writes them. Finder is driven through AppleScript and its rows read through the accessibility interface, so the app this runs in needs permission for both
	let root = arguments[2].replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "\"", with: "\\\"")
	let source = """
	tell application "Finder"
		set folderList to every folder of folder (POSIX file "\(root)" as alias)
		set w to make new Finder window to (item 1 of folderList)
		set bounds of w to {80, 60, 1000, 1400}
		activate
	end tell
	set out to ""
	repeat with f in folderList
		tell application "Finder"
			set target of w to f
			set current view of w to list view
			try
				set sort column of list view options of w to column id name column
			end try
			try
				set sort direction of column id name column of list view options of w to normal
			end try
			if (name of sort column of list view options of w as text) is not "name column" or (sort direction of column id name column of list view options of w as text) is not "normal" then error "Finder would not sort " & (name of f) & " by name"
			set folderName to name of f
		end tell
		delay 1
		tell application "System Events" to tell process "Finder"
			repeat with r in rows of outline 1 of scroll area 1 of splitter group 1 of splitter group 1 of front window
				set out to out & folderName & tab & ((value of text field 1 of UI element 1 of r) as text) & linefeed
			end repeat
		end tell
	end repeat
	tell application "Finder" to close w
	return out
	"""
	var trouble: NSDictionary?
	guard let shown = NSAppleScript(source: source)?.executeAndReturnError(&trouble).stringValue else { FileHandle.standardError.write("Finder: \(trouble ?? [:])\n".data(using: .utf8)!); exit(1) }//to standard error, which node passes through, so a refusal of permission is read where it happened
	try! shown.data(using: .utf16LittleEndian)!.write(to: URL(fileURLWithPath: arguments[3]))
	print("\(shown.split(separator: "\n").count) files")
} else {
	let collation = UserDefaults.standard.string(forKey: "AppleCollationOrder") ?? "not set"
	print("\(ProcessInfo.processInfo.operatingSystemVersionString), language \(Locale.preferredLanguages.first ?? "none"), locale \(Locale.current.identifier), list sort order \(collation)")
}
