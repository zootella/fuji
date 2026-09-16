use std::sync::OnceLock;
use objc2::runtime::{AnyObject, ClassBuilder, Sel};
use objc2::{sel, MainThreadMarker};

use crate::log;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::NSString;
use tauri::AppHandle;

/*
The menu that appears when the user right-clicks or holds fuji's icon in the dock, which is not the menu bar and shares nothing with it.

**Most of that menu is already there and fuji built none of it.** macOS fills it by itself with every open window named by its title, Options, Show All Windows, Hide and Quit. What an application may do is add its own items to the top, and the only way to do that is one delegate method: `applicationDockMenu:`, which AppKit calls each time the menu is about to open and which returns a menu. There is no property to set and no key in Info.plist. Finder's *New Finder Window* is this method and nothing else.

**Tauri does not expose it** — an open feature request since 2022, tauri#4520 — so fuji makes the call itself, which is what this module is.

**One item, and one is the right number.** This menu is reached when the application is not in front of the user, so the only commands that belong are ways to *start* something — which is why Finder offers New Finder Window and little else. New Window is the only command of that kind fuji has. Everything else it can do acts on the window in front, and there is no window in front.

**Installing the method without disturbing anyone.** tao owns the application delegate and installs it before fuji's code runs, and that delegate does not implement `applicationDockMenu:`. So fuji asks AppKit for whatever delegate is there, builds a *subclass* of its class with the method added, and re-classes the live object into it. Nothing tao wrote is modified or replaced; the subclass inherits every one of its methods and adds one it did not have. Re-classing a live object is the documented technique that key-value observing itself uses, and it is safe here for the reason it is safe there: the subclass adds no instance variables, so the object's memory layout is unchanged.

Asking AppKit for the delegate rather than naming tao's class is what keeps this from depending on tao's internals. If a later tao implements `applicationDockMenu:` of its own, fuji's override wins for the same reason any subclass does, which is worth knowing but not worth guarding: the item would be fuji's rather than tao's, and tao has no reason to add one.

**The item's target is the delegate itself.** The same subclass carries a second added method, the action, so there is no separate object to create and keep alive — AppKit already holds the delegate for the life of the process. Both methods reach fuji through one handle put aside at install time, because a method added this way is a plain C function with no room to carry anything.

**The menu is built fresh on every right-click** and handed back autoreleased, which is what the method's contract asks for. Building it once and keeping it would mean holding an AppKit object in a Rust static, which is the kind of thing that wants a lock and buys nothing: this runs when a person's mouse is at rest on a dock icon.
*/

static DOCK_APP: OnceLock<AppHandle> = OnceLock::new();//the one handle both added methods reach fuji through; a runtime-added method is a bare C function and can carry nothing of its own

/// Give fuji's dock icon its own New Window item; call once during setup, on macOS
pub fn dock_install(app: &AppHandle) {
	if DOCK_APP.set(app.clone()).is_err() { return }//already installed, and a second subclass of the same name would fail to register anyway

	let Some(marker) = MainThreadMarker::new() else { return };
	let application = NSApplication::sharedApplication(marker);
	let Some(delegate) = application.delegate() else { return };//no delegate yet, which would mean this ran impossibly early

	let object: &AnyObject = delegate.as_ref();//tao's delegate as a plain object, which is all that is wanted here: its class, and something to re-class
	let original = object.class();

	let Some(mut builder) = ClassBuilder::new(c"FujiDockDelegate", original) else { return };//a name already taken means this ran twice, and the guard above should have stopped that
	unsafe {//each of these adds a method the superclass does not have, so nothing tao wrote is replaced
		builder.add_method(sel!(applicationDockMenu:), dock_menu as extern "C" fn(_, _, _) -> _);
		builder.add_method(sel!(fujiDockNewWindow:), dock_new_window as extern "C" fn(_, _, _));
	}
	let class = builder.register();

	let previous = unsafe { AnyObject::set_class(object, class) };//objc2's own wrapper rather than the raw call, and it hands back the class it replaced
	if !std::ptr::eq(previous, original) { log::log("⭕ dock: the delegate's class changed under us, so the dock menu may not be installed") }//objc2 asks callers to check this, because somebody else re-classing the same object concurrently is the one way it goes wrong
}

//AppKit asks for the menu every time the user opens it, and takes it autoreleased
extern "C" fn dock_menu(this: &AnyObject, _command: Sel, _sender: &AnyObject) -> *mut NSMenu {
	let Some(marker) = MainThreadMarker::new() else { return std::ptr::null_mut() };
	let menu = NSMenu::new(marker);
	let item = unsafe {
		NSMenuItem::initWithTitle_action_keyEquivalent(
			marker.alloc(),
			&NSString::from_str("New Window"),
			Some(sel!(fujiDockNewWindow:)),
			&NSString::from_str(""),//no key equivalent: the dock menu is a mouse gesture, and ⌘N already lives in the menu bar
		)
	};
	unsafe { item.setTarget(Some(this)) };//the delegate answers it, because the delegate is where the action below was added
	menu.addItem(&item);
	Retained::autorelease_return(menu)//the contract is a +0 object; keeping one in a rust static instead would want a lock and buy nothing
}

//the item was chosen, which is the only thing fuji's dock menu can do
extern "C" fn dock_new_window(_this: &AnyObject, _command: Sel, _sender: &AnyObject) {
	if let Some(app) = DOCK_APP.get() { crate::window::window_open(app, vec![]) }//the same call ⌘N and a dock click on an empty fuji already make
}
