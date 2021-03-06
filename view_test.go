package smash

import (
	"github.com/evmar/smash/keys"
	"github.com/evmar/smash/ui/fake"

	"github.com/evmar/gocairo/cairo"
)

type testViewHost struct {
	win Window
}

func NewTestViewHost(ui *fake.UI) *testViewHost {
	font := NewMonoFont()
	font.fakeMetrics()
	return &testViewHost{
		win: Window{
			ui:   ui,
			font: font,
		},
	}
}

func (tv *testViewHost) GetWindow() *Window {
	return &tv.win
}
func (tv *testViewHost) Draw(cr *cairo.Context) {}
func (tv *testViewHost) Key(key keys.Key) bool {
	return false
}
func (tv *testViewHost) Scroll(dy int) {}
func (tv *testViewHost) Dirty()        {}
func (tv *testViewHost) Enqueue(f func()) {
	tv.win.ui.Enqueue(f)
}
