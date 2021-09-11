use enclose::enclose;
use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
use serde::Serialize;
//import refcell
use std::cell::RefCell;
//import arc
use std::sync::Arc;
//import mutex
use std::sync::Mutex;
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
//TODO serialize and save these to dot file
///a single action of the mouse
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct MouseAction {
    //whatever the precision of the monitor is
    pub location: (i32, i32),
    pub is_clicked: bool,
}

//TODO: function that takes a function and passes to bind but saves
//      press as a log for the mouse action
// fn fn_rec()

//TODO: need to stop using Arc<Mutex<T>> and use lifetimes
//TODO: ensure this doesnt deadlock on Mutex forever. want to
//      create an access queue on the mutex when this occurs
pub trait BindRecord {
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
        self,
        f: F,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    );
    fn bind_release_rec(self, f: fn() -> (), history: Arc<Mutex<RefCell<Vec<MouseAction>>>>);
}
impl BindRecord for KeybdKey {
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
        self,
        f: F,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
        self.bind(enclose!((self=>cur_key, history)move || {
            f();

            let mut is_clicked = false;
            //TODO: is this the right way to do this cmp?
            if u64::from(cur_key.to_owned()) == u64::from(KeybdKey::MouseKeyMiddle) {
                //this was a click
                is_clicked = true;
            }
            let position = inputbot::MouseCursor::get_pos_abs();

            //now get cursor location
            let cur_action = MouseAction {
                location: position,
                is_clicked: is_clicked,
            };
            history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
            //print history using inspect
            //TODO: write less and put a cap on in memory size at some point
            // let mut file = File::create("rat_nest").unwrap();
            let mut file = OpenOptions::new().write(true).append(true).open("rat_nest").unwrap();
            file.write_all(format!("{},{},{} \n", cur_action.location.0, cur_action.location.1, cur_action.is_clicked as u8)
            .as_bytes()).unwrap();
        }));
    }

    fn bind_release_rec(self, f: fn(), history: Arc<Mutex<RefCell<Vec<MouseAction>>>>) {
        self.bind(enclose!((history) move || {
            f();

            let position = inputbot::MouseCursor::get_pos_abs();

            //now get cursor location
            let cur_action = MouseAction {
                location: position,
                is_clicked: false,
            };
            history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
        }));
    }
}
