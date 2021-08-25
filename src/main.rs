use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
//use tensorflow::ops::Enter;
// use these to feed to Rat_Tunnel network and animate
// motions such as lines to track tunnel cursor telepor
use x11::xlib::{XGetImage, XPutImage};
use x11::{xinput2, xlib};
//import crate for delay
use std::collections::HashMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use std::{self, primitive};
//import box
use enclose::enclose;
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

//a single action of the mouse
//TODO serialize and save these to dot file
struct Mouse_Action {
    //whatever the precision of the monitor is
    location: (i64, i64),
    is_clicked: bool,
}

trait MoveRat {
    fn move_rat(
        self,
        is_numlock_on: Arc<Mutex<RefCell<Box<bool>>>>,
        is_fast: Arc<Mutex<RefCell<Box<bool>>>>,
        is_slow: Arc<Mutex<RefCell<Box<bool>>>>,
        fast_speed: u64,
        medium_speed: u64,
        slow_speed: u64,
        mode_keypad: KeybdKey,
        mode_arrow: KeybdKey,
        x: i32,
        y: i32,
    );
}
impl MoveRat for KeybdKey {
    fn move_rat(
        self,
        is_numlock_on: Arc<Mutex<RefCell<Box<bool>>>>,
        is_fast: Arc<Mutex<RefCell<Box<bool>>>>,
        is_slow: Arc<Mutex<RefCell<Box<bool>>>>,
        fast_speed: u64,
        medium_speed: u64,
        slow_speed: u64,
        mode_keypad: KeybdKey,
        mode_arrow: KeybdKey,
        x: i32,
        y: i32,
    ) {
        self.bind(enclose!((is_numlock_on, is_fast) move || {
            if *is_numlock_on.lock().unwrap().borrow().clone() {
                while self.is_pressed() {
                    let is_slow = *is_slow.lock().unwrap().borrow().clone();
                    let is_fast = *is_fast.lock().unwrap().borrow().clone();
                    //TODO: slow mode with mixing using plus key
                    //move up with fast or slow speed
                    if is_fast && is_slow {
                        //move up with fast speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros((medium_speed-fast_speed)/2 as u64)); //TODO
                    }else if is_fast {
                        //move up with slow speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(fast_speed as u64));
                    }
                    else if is_slow {
                        //move up with slow speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(slow_speed as u64));
                    }
                    else {
                        //move up with medium speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(medium_speed as u64));
                    }
                }
            }
            //TODO: else .press(); arrow key or digit
        }));
    }
}
fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut args = args
        .into_iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<i32>>();
    let fast_speed = args.pop().unwrap();
    let medium_speed = args.pop().unwrap();
    let slow_speed = args.pop().unwrap();
    let click_speed = args.pop().unwrap();

    //the history buffer of mouse clicks and current location
    // let mut history = vec![];
    //the stored procedures of the mouse where keys are 1-9 and values are
    //vectors of postitions and possible clicks
    // let mut robots = HashMap::new();

    //using Arc Mutex Refcell isnt ideal but its still fast and NKRO complete. would prefer lifetime only but needs sync
    let left_click_toggle = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

    //create is_fast for up down left right and all diagonals
    //TODO: NKRO locks this on mutex: if two or more buttons are pressed at the same time as is_fast is toggled.
    //      not a big deal.
    let is_fast = Arc::new(Mutex::new(RefCell::new(Box::new(false))));
    let is_slow = Arc::new(Mutex::new(RefCell::new(Box::new(false))));

    // TODO: force this to sync with numlock on initialization
    let is_numlock_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

    //  Num_Lock can't keep up so we need to write our own stateful modes using different toggle keys
    let mut awaits = vec![];

    //KP_Home
    //KP_Up
    //KP_Prior
    //KP_Subtract
    //KP_Left
    //KP_Begin
    //KP_Right
    //KP_Add
    //KP_End
    //KP_Down
    //KP_Next
    //KP_Insert
    //KP_Delete
    for i in 0..13 {
        let command_str = format!(
            "keycode {} =
        {}",
            i + 79,
            i + 900
        );
        awaits.push(
            std::process::Command::new("xmodmap")
                .args(&["-e", command_str.as_str()])
                .spawn(), //.output(),
        );
    }
    //TODO: implement Enter key as slow mode with fast mode mixing
    //TODO this may be marginal with above loop check during implementation
    //TODO: this is a hack to prevent keypad from sending
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 104 = 1000"#]) //TODO: wrong
            .spawn(),
    );
    //asterisk
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 63 = 914 914"#])
            .spawn(),
    );
    //forward slash
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 106 = 915"#]) //TODO: 98
            .spawn(),
    );
    // enter
    // awaits.push(
    //     std::process::Command::new("xmodmap")
    //         .args(&["-e", r#"keycode 104 = 916 916"#])
    //         .spawn(),
    // );
    //TODO: ?
    //also remap numlock since NKRO numpads dont arrive in order at usb
    //hub causing entries to not have numlock signal prepended
    // awaits.push(
    //     std::process::Command::new("xmodmap")
    //         .args(&["-e", r#"keycode 77=916 916"#])
    //         .spawn(),
    // );
    awaits.into_iter().for_each(|x| {
        x.unwrap();
    });

    MouseKeyUp.move_rat(
        // cloning here is weird but doesnt really matter since this is config
        // and i'll take what I can get from the borrow checker
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad8Key,
        UpKey,
        0,
        -1,
    );
    MouseKeyDown.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad2Key,
        DownKey,
        0,
        1,
    );
    MouseKeyLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad4Key,
        LeftKey,
        -1,
        0,
    );
    MouseKeyRight.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad6Key,
        RightKey,
        1,
        0,
    );
    MouseKeyUpperLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad7Key,
        //TODO: this should be up and left at the same time
        UpKey,
        -1,
        -1,
    );
    MouseKeyUpperRight.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad9Key,
        UpKey,
        1,
        -1,
    );
    MouseKeyLowerRight.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad3Key,
        DownKey,
        1,
        1,
    );
    MouseKeyLowerLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        is_slow.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        Numpad1Key,
        DownKey,
        -1,
        1,
    );

    //TODO: fix this naming in input keycode
    NumpadPlusKey.bind(
        enclose!((is_numlock_on=>is_numlock_on_click_toggle,left_click_toggle=> right_click_toggle) move || {
            if *is_numlock_on_click_toggle.lock().unwrap().borrow().clone() {
                MouseButton::RightButton.press();
                sleep(Duration::from_micros(click_speed as u64));
                MouseButton::RightButton.release();

                right_click_toggle
                    .to_owned()
                    .lock()
                    .unwrap()
                    .replace(Box::new(false));
            }
        }),
    );
    //Numpad1Key.bind(|| {
    //TODO: medium speed for held key check should be a config option?
    MouseKeyFast.bind(enclose!((is_numlock_on=>is_numlock_on_fast)move || {
            //set fast speed
            is_fast.to_owned().lock().unwrap().replace(Box::new(true));
            // fast is not modal for ergonomics.
            while MouseKeyFast.is_pressed() {
                sleep(Duration::from_micros(medium_speed as u64));
                continue;
            }
            is_fast.to_owned().lock().unwrap().replace(Box::new(false));
    }));

    //same as fast for slow using enter key
    MouseKeySlow.bind(enclose!((is_slow)move || {
        is_slow.to_owned().lock().unwrap().replace(Box::new(true));
    }));
    MouseKeySlow.release_bind(enclose!((is_slow) move||{
        is_slow.to_owned().lock().unwrap().replace(Box::new(false));
    }));

    //TODO: numlock key is unreliable due to nkro and speed
    //toggle is numlock on each time num lock key is pressed
    // MouseKeyActivate.bind(move || {
    // NumLockKey.bind(enclose!((is_numlock_on) move || {
    //     let cur_value = **is_numlock_on.clone().lock().unwrap().borrow();
    //     is_numlock_on
    //         .to_owned()
    //         .lock()
    //         .unwrap()
    //         .replace(Box::new(!cur_value));
    // }));

    //Numpad5Key.bind(|| {
    MouseKeyMiddle.bind(
        enclose!((is_numlock_on=>is_numlock_on_middle, left_click_toggle) move || {
            if *is_numlock_on_middle.lock().unwrap().borrow().clone() {
                //toggle left click
                //let cur_value = *left_click_active.lock().unwrap().borrow().clone();
                //if cur_value.clone() {
                MouseButton::LeftButton.press();
                sleep(Duration::from_micros(click_speed as u64));
                MouseButton::LeftButton.release();
                left_click_toggle
                    .to_owned()
                    .lock()
                    .unwrap()
                    .replace(Box::new(true));
                //} else {
                //    MouseButton::RightButton.press();
                //    sleep(Duration::from_micros(10));
                //    MouseButton::RightButton.release();
                //}
            }
        }),
    );
    //TODO: change these names in input
    MouseKeyClickToggle.bind(
        enclose!((is_numlock_on=>is_numlock_on_plus,left_click_toggle=>left_click_hold)move || {
            if **is_numlock_on_plus.lock().unwrap().borrow() {
                //hold left click, released by another 5 left click
                if *left_click_hold.lock().unwrap().borrow().clone() {
                    MouseButton::LeftButton.press();
                } else {
                    //right
                    MouseButton::RightButton.press();
                }
            }
        }),
    );

    handle_input_events();
}
