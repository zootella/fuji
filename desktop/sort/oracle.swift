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
} else {
	let collation = UserDefaults.standard.string(forKey: "AppleCollationOrder") ?? "not set"
	print("\(ProcessInfo.processInfo.operatingSystemVersionString), language \(Locale.preferredLanguages.first ?? "none"), locale \(Locale.current.identifier), list sort order \(collation)")
}
