use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
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
    let left_click_active = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let left_click_counted = left_click_active.clone();

    let is_fast = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    //create create is fast for up down left right and all diagonals
    let is_up_fast = is_fast.clone();
    let is_down_fast = is_fast.clone();
    let is_left_fast = is_fast.clone();
    let is_right_fast = is_fast.clone();
    let is_up_left_fast = is_fast.clone();
    let is_up_right_fast = is_fast.clone();
    let is_down_left_fast = is_fast.clone();
    let is_down_right_fast = is_fast.clone();
    //TODO: block keys as well to prevent key spam. for now just use numpad toggle

    //we're going to use the numb pads on each left right top and bottom to move the mouse abs up down left and right
    //Numpad8Key.bind(|| {
    UpKey.bind(move || {
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
    });
    //Numpad2Key.bind(|| {
    DownKey.bind(move || {
        //Numpad2Key.bind(|| {
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
    });
    //Numpad4Key.bind(|| {
    LeftKey.bind(move || {
        //Numpad4Key.bind(|| {
        //move left with fast or slow speed
        while Numpad4Key.is_pressed() {
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
    });
    // rewrite the above comment it outlines of code just like up key left key and down key
    RightKey.bind(move || {
        //Numpad6Key.bind(|| {
        //move right with fast or slow speed
        while Numpad6Key.is_pressed() {
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
    });
    //Numpad5Key.bind(|| {
    HomeKey.bind(move || {
        //Numpad7Key.bind(|| {
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
    });
    //Numpad1Key.bind(|| {
    Numpad1Key.bind(move || {
        //Numpad1Key.bind(|| {
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
    });
    //Numpad3Key.bind(|| {
    Numpad3Key.bind(move || {
        //Numpad3Key.bind(|| {
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
    });
    Numpad9Key.bind(move || {
        //Numpad9Key.bind(|| {
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
    });

    // num pad five toggles either left or right click and forward slash an asterisk respectively
    // toggle whether left or right click is set for num pad five
    // TODO: this should be forward and asterisk on the num pad to prevent accidental toggling
    DeleteKey.bind(move || {
        let cur_value = **left_click_counted.to_owned().lock().unwrap().borrow();
        left_click_counted
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    });
    Numpad5Key.bind(move || {
        if *left_click_active.lock().unwrap().borrow().clone() == true {
            //left click
            //we're going to use the num pad 0 to toggle holding down the left mouse button
            //toggle holding left mouse button
            LeftButton.press();
            //delay 10 ms
            sleep(Duration::from_millis(10));
            LeftButton.release();
        } else {
            RightButton.press();
            sleep(Duration::from_millis(10));
            RightButton.release();
        }
    });
    // numpad enter symbol toggles fast speed
    InsertKey.bind(move || {
        //set fast speed
        let cur_value = **is_fast.clone().lock().unwrap().borrow();
        is_fast
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    });

    //TODO: hold mouse toggle
    //Numpad0Key.bind(|| {

    handle_input_events();
}
//TODO: feature for speed and acceleration etc. has to be a feature so it can be user defined easily
