use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
use x11::{xinput2, xlib};
//import crate for delay
use std::env;
use std::thread::sleep;
use std::time::Duration;
//import box
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut args = args
        .into_iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<i32>>();
    let fast_speed = args.pop().unwrap();
    let slow_speed = args.pop().unwrap();
    let move_frequency = args.pop().unwrap() as u64;

    //using Arc Mutex Refcell isnt ideal but its still fast and NKRO complete. would prefer lifetime only but needs sync
    //whether 5 left or right clicks
    let left_click_active = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let left_click_counted = left_click_active.clone();

    //create create is fast for up down left right and all diagonals
    let is_fast = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let is_up_fast = is_fast.clone();
    let is_down_fast = is_fast.clone();
    let is_left_fast = is_fast.clone();
    let is_right_fast = is_fast.clone();
    let is_up_left_fast = is_fast.clone();
    let is_up_right_fast = is_fast.clone();
    let is_down_left_fast = is_fast.clone();
    let is_down_right_fast = is_fast.clone();

    //TODO: use num lock key with mutex instead of relying on xlib toggle
    let is_numlock_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let is_numlock_on_up = is_numlock_on.clone();
    let is_numlock_on_down = is_numlock_on.clone();
    let is_numlock_on_left = is_numlock_on.clone();
    let is_numlock_on_right = is_numlock_on.clone();
    let is_numlock_on_up_left = is_numlock_on.clone();
    let is_numlock_on_up_right = is_numlock_on.clone();
    let is_numlock_on_down_left = is_numlock_on.clone();
    let is_numlock_on_down_right = is_numlock_on.clone();

    //TODO: block keys as well to prevent key spam.
    //TODO: remap keys to unused keys from within x to prevent typing to entry while MouseKeying
    //      consider xmodmap -e "remove <key> <key>" to remap keys to unused keys

    //Numpad8Key.bind(|| {
    UpKey.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad8Key.is_pressed() {
                //move up with fast or slow speed
                if *is_up_fast.lock().unwrap().borrow().clone() {
                    //move up with fast speed
                    MouseCursor::move_abs(0, -fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move up with slow speed
                    MouseCursor::move_abs(0, -slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad2Key.bind(|| {
    DownKey.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad2Key.is_pressed() {
                //move down with fast or slow speed
                if *is_down_fast.lock().unwrap().borrow().clone() {
                    //move down with fast speed
                    MouseCursor::move_abs(0, fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move down with slow speed
                    MouseCursor::move_abs(0, slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad4Key.bind(|| {
    LeftKey.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad4Key.is_pressed() {
                //move left with fast or slow speed
                if *is_left_fast.lock().unwrap().borrow().clone() {
                    //move left with fast speed
                    MouseCursor::move_abs(-fast_speed, 0);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move left with slow speed
                    MouseCursor::move_abs(-slow_speed, 0);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad6Key.bind(|| {
    RightKey.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad6Key.is_pressed() {
                //move right with fast or slow speed
                if *is_right_fast.lock().unwrap().borrow().clone() {
                    //move right with fast speed
                    MouseCursor::move_abs(fast_speed, 0);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move right with slow speed
                    MouseCursor::move_abs(slow_speed, 0);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad7Key.bind(|| {
    HomeKey.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad7Key.is_pressed() {
                //move up left with fast or slow speed
                if *is_up_left_fast.lock().unwrap().borrow().clone() {
                    //move up left with fast speed
                    MouseCursor::move_abs(-fast_speed, -fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move up left with slow speed
                    MouseCursor::move_abs(-slow_speed, -slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad9Key.bind(|| {
    Numpad9Key.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad9Key.is_pressed() {
                //move up right with fast or slow speed
                if *is_up_right_fast.lock().unwrap().borrow().clone() {
                    //move up right with fast speed
                    MouseCursor::move_abs(fast_speed, -fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move up right with slow speed
                    MouseCursor::move_abs(slow_speed, -slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad3Key.bind(|| {
    Numpad3Key.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad3Key.is_pressed() {
                //move down right with fast or slow speed
                if *is_down_right_fast.lock().unwrap().borrow().clone() {
                    //move down right with fast speed
                    MouseCursor::move_abs(fast_speed, fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move down right with slow speed
                    MouseCursor::move_abs(slow_speed, slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });
    //Numpad1Key.bind(|| {
    Numpad1Key.bind(move || {
        if NumLockKey.is_toggled() {
            while Numpad1Key.is_pressed() {
                //move down left with fast or slow speed
                if *is_down_left_fast.lock().unwrap().borrow().clone() {
                    //move down left with fast speed
                    MouseCursor::move_abs(-fast_speed, fast_speed);
                    sleep(Duration::from_millis(move_frequency));
                } else {
                    //move down left with slow speed
                    MouseCursor::move_abs(-slow_speed, slow_speed);
                    sleep(Duration::from_millis(move_frequency));
                }
            }
        }
    });

    DeleteKey.bind(move || {
        if NumLockKey.is_toggled() {
            //toggle whether left click is counted for num pad five
            let cur_value = **left_click_counted.to_owned().lock().unwrap().borrow();
            left_click_counted
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(!cur_value));
        }
    });
    //Numpad1Key.bind(|| {
    InsertKey.bind(move || {
        if NumLockKey.is_toggled() {
            //set fast speed
            let cur_value = **is_fast.clone().lock().unwrap().borrow();
            is_fast
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(!cur_value));
        }
    });
    //Numpad5Key.bind(|| {
    Numpad5Key.bind(move || {
        if NumLockKey.is_toggled() {
            //toggle left click
            let cur_value = *left_click_active.lock().unwrap().borrow().clone();
            if cur_value.clone() {
                MouseButton::LeftButton.press();
                sleep(Duration::from_millis(10));
                MouseButton::LeftButton.release();
            } else {
                MouseButton::RightButton.press();
                sleep(Duration::from_millis(10));
                MouseButton::RightButton.release();
            }
        }
    });
    NumpadPlusKey.bind(move || {
        if NumLockKey.is_toggled() {
            //hold left click, released by another 5 left click
            MouseButton::LeftButton.press();
        }
    });

    //Numpad1Key.bind(move || {

    //TODO: hold mouse toggle
    //Numpad0Key.bind(|| {

    handle_input_events();
}
//TODO: feature for speed and acceleration etc. has to be a feature so it can be user defined easily
