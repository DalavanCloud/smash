extern crate cairo;
extern crate gdk;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use prompt::Prompt;
use term::Term;
use view;
use view::Layout;

pub struct LogEntry {
    prompt: Prompt,
    term: RefCell<Option<Term>>,
    layout: Cell<Layout>,
}

impl LogEntry {
    pub fn new(dirty: Rc<Fn()>, font_extents: cairo::FontExtents, done: Box<Fn()>) -> Rc<LogEntry> {
        let le = Rc::new(LogEntry {
            prompt: Prompt::new(dirty.clone()),
            term: RefCell::new(None),
            layout: Cell::new(Layout::new()),
        });

        let accept_cb = {
            // The accept callback from readline can potentially be
            // called multiple times, but we only want create a
            // terminal once.  Capture all the needed state in a
            // moveable temporary.
            let mut once = Some((le.clone(), dirty, font_extents, done));
            Box::new(move |str: &str| {
                if let Some(once) = once.take() {
                    let text = String::from(str);
                    view::add_task(move || {
                        let (le, dirty, font_extents, done) = once;
                        *le.term.borrow_mut() =
                            Some(Term::new(dirty, font_extents, &[&text], done));
                    })
                }
            })
        };
        le.prompt.set_accept_cb(accept_cb);
        le
    }
}

impl view::View for LogEntry {
    fn draw(&self, cr: &cairo::Context, focus: bool) {
        if let Some(ref term) = *self.term.borrow() {
            self.prompt.draw(cr, false);
            cr.save();
            let height = self.prompt.get_layout().height as f64;
            cr.translate(0.0, height);
            term.draw(cr, focus);
            cr.restore();
        } else {
            self.prompt.draw(cr, focus);
        }
    }

    fn key(&self, ev: &gdk::EventKey) {
        if let Some(ref term) = *self.term.borrow() {
            term.key(ev);
        } else {
            self.prompt.key(ev);
        }
    }

    fn relayout(&self, cr: &cairo::Context, space: Layout) -> Layout {
        let mut layout = self.prompt.relayout(cr, space);
        if let Some(ref term) = *self.term.borrow() {
            let tlayout = term.relayout(cr,
                                        Layout {
                                            width: space.width,
                                            height: space.height - layout.height,
                                        });
            layout = layout.add(tlayout.width, tlayout.height);
        }
        self.layout.set(layout);
        layout
    }
    fn get_layout(&self) -> Layout {
        self.layout.get()
    }
}

pub struct Log {
    entries: RefCell<Vec<Rc<LogEntry>>>,
    dirty: Rc<Fn()>,
    font_extents: cairo::FontExtents,
    layout: Cell<Layout>,
}

impl Log {
    pub fn new(dirty: Rc<Fn()>, font_extents: &cairo::FontExtents) -> Rc<Log> {
        let log = Rc::new(Log {
            entries: RefCell::new(Vec::new()),
            dirty: dirty,
            font_extents: font_extents.clone(),
            layout: Cell::new(Layout::new()),
        });
        Log::new_entry(&log);
        log
    }

    pub fn new_entry(log: &Rc<Log>) {
        let entry = {
            let log = log.clone();
            LogEntry::new(log.dirty.clone(),
                          log.font_extents,
                          Box::new(move || {
                              Log::new_entry(&log);
                          }))
        };
        log.entries.borrow_mut().push(entry);
    }
}

impl view::View for Log {
    fn draw(&self, cr: &cairo::Context, focus: bool) {
        let entries = self.entries.borrow();
        cr.save();
        for (i, entry) in entries.iter().enumerate() {
            let last = i == entries.len() - 1;
            entry.draw(cr, focus && last);
            cr.translate(0.0, entry.get_layout().height as f64);
        }
        cr.restore();
    }
    fn key(&self, ev: &gdk::EventKey) {
        let entries = self.entries.borrow();
        entries[entries.len() - 1].key(ev);
    }
    fn relayout(&self, cr: &cairo::Context, space: Layout) -> Layout {
        let entries = self.entries.borrow();
        let mut height = 0;
        for entry in &*entries {
            let entry_layout = entry.relayout(cr, space.add(0, -height));
            height += entry_layout.height;
        }
        self.layout.set(Layout {
            width: space.width,
            height: height,
        });
        self.layout.get()
    }
    fn get_layout(&self) -> Layout {
        self.layout.get()
    }
}
