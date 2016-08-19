extern crate cairo;
extern crate gdk;
extern crate gtk;
use gdk::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;


// Following
// https://github.com/google/xi-editor/commit/43abeb4adc36a60090e44dc1c74f5eabbfc77b7f
// a workaround for no Box<FnOnce()>.
trait Task {
    fn call(self: Box<Self>);
}

impl<F: FnOnce()> Task for F {
    fn call(self: Box<F>) {
        self();
    }
}

thread_local!(static TASKS: RefCell<Vec<Box<Task>>> = RefCell::new(Vec::new()));

pub fn add_task<F: FnOnce() + 'static>(task: F) {
    TASKS.with(|tasks| {
        tasks.borrow_mut().push(Box::new(task));
    });
    gtk::idle_add(run_tasks);
}

fn run_tasks() -> gtk::Continue {
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        for t in tasks.drain(..) {
            t.call();
        }
    });
    Continue(false)
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Layout {
    pub width: i32,
    pub height: i32,
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            width: 0,
            height: 0,
        }
    }
    pub fn add(&self, w: i32, h: i32) -> Layout {
        let layout = Layout {
            width: self.width + w,
            height: self.height + h,
        };
        if layout.width < 0 || layout.height < 0 {
            println!("layout underflow: {:?}", layout);
        }
        layout
    }
}

pub trait View {
    fn draw(&self, cr: &cairo::Context, focus: bool);
    fn key(&self, ev: &gdk::EventKey);
    fn relayout(&self, _cr: &cairo::Context, _space: Layout) -> Layout {
        self.get_layout()
    }
    fn get_layout(&self) -> Layout;
}

pub struct NullView {}
impl View for NullView {
    fn draw(&self, _cr: &cairo::Context, _focus: bool) {}
    fn key(&self, _ev: &gdk::EventKey) {}
    fn get_layout(&self) -> Layout {
        Layout::new()
    }
}

pub struct Win {
    pub dirty_cb: Rc<Fn()>,
    pub gtkwin: gtk::Window,
    pub child: RefCell<Rc<View>>,
}

impl Win {
    pub fn new() -> Rc<Win> {
        let gtkwin = gtk::Window::new(gtk::WindowType::Toplevel);
        gtkwin.set_default_size(400, 200);
        gtkwin.set_app_paintable(true);
        gtkwin.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let draw_pending = Rc::new(Cell::new(false));
        let dirty_cb = {
            let gtkwin = gtkwin.clone();
            let draw_pending = draw_pending.clone();
            Rc::new(move || {
                if draw_pending.get() {
                    println!("debounce dirty");
                    return;
                }
                draw_pending.set(true);
                gtkwin.queue_draw();
            })
        };

        let win = Rc::new(Win {
            dirty_cb: dirty_cb,
            gtkwin: gtkwin.clone(),
            child: RefCell::new(Rc::new(NullView {})),
        });

        {
            let win = win.clone();
            gtkwin.connect_draw(move |_, cr| {
                let child = win.child.borrow();
                child.relayout(cr,
                               Layout {
                                   width: 600,
                                   height: 400,
                               });
                child.draw(cr, true);
                draw_pending.set(false);
                Inhibit(false)
            });
        }
        {
            let win = win.clone();
            gtkwin.connect_key_press_event(move |_, ev| {
                let child = win.child.borrow();
                child.key(ev);
                Inhibit(false)
            });
        }

        win
    }

    pub fn create_cairo(&self) -> cairo::Context {
        self.gtkwin.realize();
        let gdkwin = self.gtkwin.get_window().unwrap();
        cairo::Context::create_from_window(&gdkwin)
    }

    pub fn resize(&self, width: i32, height: i32) {
        self.gtkwin.resize(width, height);
    }

    pub fn show(&self) {
        self.gtkwin.show();
    }
}

pub fn is_modifier_key_event(ev: &gdk::EventKey) -> bool {
    match ev.get_keyval() {
        gdk::enums::key::Caps_Lock |
        gdk::enums::key::Control_L |
        gdk::enums::key::Control_R |
        gdk::enums::key::Shift_L |
        gdk::enums::key::Shift_R |
        gdk::enums::key::Alt_L |
        gdk::enums::key::Alt_R |
        gdk::enums::key::Meta_L |
        gdk::enums::key::Meta_R => true,
        _ => false,
    }
}

pub fn init() {
    gtk::init().unwrap();
}

pub fn main() {
    gtk::main();
}

pub fn quit() {
    gtk::main_quit();
}