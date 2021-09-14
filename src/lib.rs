use enclose::enclose;
use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
use serde::Serialize;
//import refcell
use std::cell::{Cell, RefCell};
//import arc
use std::sync::Arc;
//import mutex
use std::sync::Mutex;
//import thread
use std::thread::spawn;
//import AtomicBool
use std::sync::atomic::{AtomicBool, Ordering};
// use tensorflow::ops::{Assign, Const, MatMul, Placeholder, Variable};
// use tensorflow::{Graph, Session, Tensor};

// //TODO: implement and moddify the example codes first from Rust libtorch
// for rat brain
use serde_derive::{Deserialize, Serialize};
use x11::xlib::Mod1MapIndex;

//import file for writting MouseActions
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

///a single action of the mouse for data collection
///used in machine learning algorithms
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct MouseAction {
    //whatever the precision of the monitor is
    pub location: (i32, i32),
    pub is_clicked: bool,
    //whether the fast or slow mode is used
    pub is_fast: bool,
    pub is_slow: bool,
    pub is_rat_on: bool,
}

//TODO: need to stop using Arc<Mutex<T>> and use lifetimes
//TODO: ensure this doesnt deadlock on Mutex forever. want to
//      create an access queue on the mutex when this occurs

///extension trait to record each keyboard event with a proxy inheritence
pub trait BindRecord {
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
        self,
        f: F,
        is_fast: Arc<AtomicBool>,
        is_slow: Arc<AtomicBool>,
        is_rat_on: Arc<AtomicBool>,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    );
    fn bind_release_rec<F: FnOnce() + Send + Sync + 'static + Copy>(
        self,
        f: F,
        is_fast: Arc<AtomicBool>,
        is_slow: Arc<AtomicBool>,
        is_rat_on: Arc<AtomicBool>,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    );
}
impl BindRecord for KeybdKey {
    ///binds a function to a key press and also records
    ///keypress as a mouse action in the history buffer
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
        self,
        f: F,
        is_fast: Arc<AtomicBool>,
        is_slow: Arc<AtomicBool>,
        is_rat_on: Arc<AtomicBool>,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
        self.bind(enclose!((self=>cur_key, is_fast, is_slow,is_rat_on, history)move || {
            f();

            //TODO: clean this up. spawn logger so it is async to
            //      key timing logic on the stack
            spawn(enclose!((cur_key, is_fast, is_slow,is_rat_on, history)move || {
            let mut is_clicked = false;
            //TODO: is this the right way to do this cmp?
            if u64::from(cur_key.to_owned()) == u64::from(KeybdKey::MouseKeyMiddle) {
                //this was a click
                is_clicked = true;
            }
            //TODO: this locks the mutex but ends up giving the last position
            //      on release (last press) this yields succint data and
            //      happens to be what we want but is a hack.
            let position = inputbot::MouseCursor::get_pos_abs();

            //now get cursor location
            //TODO: this causes some mutex contention RWLock would be better
            //      if possible since >1 read request for sparse 1 event
            //      write request
            let cur_action = MouseAction {
                location: position,
                is_clicked: is_clicked,
                is_fast: is_fast.load(Ordering::SeqCst),
                is_slow: is_slow.load(Ordering::SeqCst),
                is_rat_on: is_rat_on.load(Ordering::SeqCst),
            };
            history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
            //print history using inspect
            //TODO: write less and put a cap on in memory size at some point
            // let mut file = File::create("rat_nest").unwrap();
            //TODO: also log fast and slow modes
            let mut file = OpenOptions::new().write(true).append(true).open("rat_nest").unwrap();
            file.write_all(format!("{},{},{},{},{},{} \n", cur_action.location.0, cur_action.location.1, cur_action.is_clicked as u8, cur_action.is_fast as u8, cur_action.is_slow as u8, cur_action.is_rat_on as u8).as_bytes()).unwrap();
            }));
        }));
    }

    ///bind a function to be called when the key is released
    ///and also store the action in the history buffer
    fn bind_release_rec<F: FnOnce() + Send + Sync + 'static + Copy>(
        self,
        f: F,
        is_fast: Arc<AtomicBool>,
        is_slow: Arc<AtomicBool>,
        is_rat_on: Arc<AtomicBool>,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
        self.bind(enclose!((history) move || {
           spawn(f);

            let position = inputbot::MouseCursor::get_pos_abs();

            //now get cursor location
            let cur_action = MouseAction {
                location: position,
                is_clicked: false,
                is_fast: is_fast.load(Ordering::SeqCst),
                is_slow: is_slow.load(Ordering::SeqCst),
                is_rat_on: is_rat_on.load(Ordering::SeqCst),
            };
            history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
        }));
    }
}
