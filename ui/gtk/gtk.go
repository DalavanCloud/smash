// +build !headless

package gtk

/*
#cgo pkg-config: gtk+-3.0
#include <gtk/gtk.h>
#include "smashgtk.h"
*/
import "C"
import (
	"time"
	"unsafe"

	"github.com/evmar/gocairo/cairo"

	"github.com/evmar/smash/keys"
	"github.com/evmar/smash/ui"
)

type UI struct {
	cKey CKey
	// Functions enqueued to do work on the main goroutine.
	enqueued chan func()
}

func (ui *UI) CKey() *CKey {
	return &ui.cKey
}

type Window struct {
	cKey   CKey
	gtkWin *C.GtkWidget
	// Store the delegate here so this interface isn't gc'd.
	delegate ui.WinDelegate

	anims map[ui.Anim]bool
}

func (win *Window) CKey() *CKey {
	return &win.cKey
}

func Init() *UI {
	C.smash_gtk_init()
	return &UI{
		enqueued: make(chan func(), 1),
	}
}

//export smashGoIdle
func smashGoIdle(key unsafe.Pointer) int {
	ui := globalPointerStore.Get(key).(*UI)
	for {
		select {
		case f := <-ui.enqueued:
			f()
		default:
			return 0 // Don't run again
		}
	}
}

func (ui *UI) Enqueue(f func()) {
	// Proxy the function to the main thread by using g_idle_add.
	// You'd be tempted to want to pass f to g_idle_add directly, but
	// (a) passing closures into C code is annoying, and (b) we need a
	// reference to the function on the Go side for GC reasons.  So
	// it's easier to just put the function in a channel that pulls it
	// back out on the other thread.
	ui.enqueued <- f
	C.g_idle_add(C.GSourceFunc(C.smash_idle_cb), globalPointerStore.Key(ui))
}

func (_ *UI) NewWindow(delegate ui.WinDelegate, toplevel bool) ui.Win {
	win := &Window{
		delegate: delegate,
		anims:    make(map[ui.Anim]bool),
	}
	ctoplevel := C.int(0)
	if toplevel {
		ctoplevel = C.int(1)
	}
	win.gtkWin = C.smash_gtk_new_window(globalPointerStore.Key(win), ctoplevel)
	return win
}

func (ui *UI) Loop() {
	C.gtk_main()
}

func (ui *UI) Quit() {
	C.gtk_main_quit()
}

func (w *Window) Dirty() {
	C.gtk_widget_queue_draw(w.gtkWin)
}

//export smashGoTick
func smashGoTick(data unsafe.Pointer) bool {
	win := globalPointerStore.Get(data).(*Window)
	// TODO: use gdk_frame_clock_get_frame_time here instead of Go time.
	now := time.Now()
	for anim := range win.anims {
		if !anim.Frame(now) {
			delete(win.anims, anim)
		}
	}
	return len(win.anims) > 0
}

func (w *Window) GetCairo() *cairo.Context {
	return cairo.WrapContext(unsafe.Pointer(C.gdk_cairo_create(C.gtk_widget_get_window(w.gtkWin))))
}

func (w *Window) SetSize(width, height int) {
	C.gtk_window_set_default_size((*C.GtkWindow)(unsafe.Pointer(w.gtkWin)),
		C.gint(width), C.gint(height))
}

func (w *Window) SetPosition(x, y int) {
	C.gtk_window_move((*C.GtkWindow)(unsafe.Pointer(w.gtkWin)),
		C.gint(x), C.gint(y))
}

func (w *Window) GetContentPosition() (int, int) {
	gdkWin := C.gtk_widget_get_window(w.gtkWin)
	var cx, cy C.gint
	C.gdk_window_get_position(gdkWin, &cx, &cy)
	return int(cx), int(cy)
}

func (w *Window) Show() {
	C.gtk_widget_show(w.gtkWin)
}

func (w *Window) Close() {
	C.gtk_widget_destroy(w.gtkWin)
}

func (w *Window) AddAnimation(anim ui.Anim) {
	if len(w.anims) == 0 {
		C.smash_start_ticks(globalPointerStore.Key(w), w.gtkWin)
	}
	w.anims[anim] = true
}

//export smashGoDraw
func smashGoDraw(winKey unsafe.Pointer, crP unsafe.Pointer) {
	win := globalPointerStore.Get(winKey).(*Window)
	cr := cairo.BorrowContext(crP)
	win.delegate.Draw(cr)
}

//export smashGoKey
func smashGoKey(winKey unsafe.Pointer, gkey *C.GdkEventKey) {
	win := globalPointerStore.Get(winKey).(*Window)

	switch gkey.keyval {
	case C.GDK_KEY_Shift_L, C.GDK_KEY_Shift_R,
		C.GDK_KEY_Control_L, C.GDK_KEY_Control_R,
		C.GDK_KEY_Alt_L, C.GDK_KEY_Alt_R,
		C.GDK_KEY_Meta_L, C.GDK_KEY_Meta_R,
		C.GDK_KEY_Super_L, C.GDK_KEY_Super_R:
		return
	}

	rune := C.gdk_keyval_to_unicode(gkey.keyval)
	key := keys.Key{}
	key.Sym = keys.Sym(rune)
	if gkey.state&C.GDK_CONTROL_MASK != 0 {
		key.Mods |= keys.ModControl
	}
	if gkey.state&C.GDK_MOD1_MASK != 0 {
		key.Mods |= keys.ModMeta
	}
	win.delegate.Key(key)
}
