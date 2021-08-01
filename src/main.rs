use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
use x11::{xinput2, xlib};
//import crate for delay
use std;
use std::collections::HashMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;
//import box
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

//a single action of the mouse
//TODO serialize and save these to dot file
struct mouse_action {
    //whatever the precision of the monitor is
    location: (i64, i64),
    is_clicked: bool,
}
fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut args = args
        .into_iter()
        .map(|x| x.parse().unwrap())
        .collect::<Vec<i32>>();
    let fast_speed = args.pop().unwrap();
    let slow_speed = args.pop().unwrap();
    let move_frequency = args.pop().unwrap() as u64;

    //the history buffer of mouse clicks and current location
    // let mut history = vec![];
    //the stored procedures of the mouse where keys are 1-9 and values are
    //vectors of postitions and possible clicks
    // let mut robots = HashMap::new();

    //using Arc Mutex Refcell isnt ideal but its still fast and NKRO complete. would prefer lifetime only but needs sync
    let left_click_active = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let left_click_toggle = left_click_active.clone();
    let left_click_hold = left_click_active.clone();

    //create is_fast for up down left right and all diagonals
    let is_fast = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let is_up_fast = is_fast.clone();
    let is_down_fast = is_fast.clone();
    let is_left_fast = is_fast.clone();
    let is_right_fast = is_fast.clone();
    let is_up_left_fast = is_fast.clone();
    let is_up_right_fast = is_fast.clone();
    let is_down_left_fast = is_fast.clone();
    let is_down_right_fast = is_fast.clone();

    //TODO: use num lock key with mutex instead of relying on xlib toggle since it misbehaves with some numpads (mine)
    let is_numlock_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
    let is_numlock_on_up = is_numlock_on.clone();
    let is_numlock_on_down = is_numlock_on.clone();
    let is_numlock_on_left = is_numlock_on.clone();
    let is_numlock_on_right = is_numlock_on.clone();
    let is_numlock_on_up_left = is_numlock_on.clone();
    let is_numlock_on_up_right = is_numlock_on.clone();
    let is_numlock_on_down_left = is_numlock_on.clone();
    let is_numlock_on_down_right = is_numlock_on.clone();
    let is_numlock_on_click_toggle = is_numlock_on.clone();
    //need to do fast plus middle
    let is_numlock_on_fast = is_numlock_on.clone();
    let is_numlock_on_plus = is_numlock_on.clone();
    let is_numlock_on_middle = is_numlock_on.clone();

    // would prefer to use the x .so but couldnt find in the SDK
    //      consider xmodmap -e "remove <key> <key>" to remap keys to unused keys
    // we are going to disable 79-89 keys to prevent typing to entry while MouseKeying
    //these are the current keymappings we will be changing by removing the KP_NUMBER entries
    // keycode  79 = KP_Home KP_7 KP_Home KP_7
    // keycode  80 = KP_Up KP_8 KP_Up KP_8
    // keycode  81 = KP_Prior KP_9 KP_Prior KP_9
    // keycode  82 = KP_Subtract KP_Subtract KP_Subtract KP_Subtract KP_Subtract KP_Subtract XF86Prev_VMode KP_Subtract KP_Subtract XF86Prev_VMode
    // keycode  83 = KP_Left KP_4 KP_Left KP_4
    // keycode  84 = KP_Begin KP_5 KP_Begin KP_5
    // keycode  85 = KP_Right KP_6 KP_Right KP_6
    // keycode  86 = KP_Add KP_Add KP_Add KP_Add KP_Add KP_Add XF86Next_VMode KP_Add KP_Add XF86Next_VMode
    // keycode  87 = KP_End KP_1 KP_End KP_1
    // keycode  88 = KP_Down KP_2 KP_Down KP_2
    // keycode  89 = KP_Next KP_3 KP_Next KP_3
    // keycode  90 = KP_Insert KP_0 KP_Insert KP_0
    // keycode  91 = KP_Delete KP_Decimal KP_Delete KP_Decimal
    // we are counting from three hundred since these values are unused in the scan codes (think virtual sockets)
    //TODO: restore default on close if no better solution found
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 79 = KP_Home 300 KP_Home 300"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 80 = KP_Up 301 KP_Up 301"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 81 = KP_Prior 302 KP_Prior 302"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 82 = KP_Subtract 303 KP_Subtract 303"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 83 = KP_Left 304 KP_Left 304"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 84 = KP_Begin 305 KP_Begin 305"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 85 = KP_Right 306 KP_Right 306"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 86 = KP_Add 307 KP_Add 307"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 87 = KP_End 308 KP_End 308"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 88 = KP_Down 309 KP_Down 309"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 89 = KP_Next 310 KP_Next 310"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 90 = KP_Insert 311 KP_Insert 311"#])
        .output()
        .unwrap();
    std::process::Command::new("xmodmap")
        .args(&["-e", r#"keycode 91 = KP_Delete 312 KP_Delete 312"#])
        .output()
        .unwrap();

    //NOTE: these should have been in a macro dont blame rust for my bad code
    //Numpad8Key.bind(|| {
    MouseKeyUp.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_up.lock().unwrap().borrow().clone() {
            while MouseKeyUp.is_pressed() {
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
    MouseKeyDown.bind(move || {
        if *is_numlock_on_down.lock().unwrap().borrow().clone() {
            while MouseKeyDown.is_pressed() {
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
    MouseKeyLeft.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_left.lock().unwrap().borrow().clone() {
            while MouseKeyLeft.is_pressed() {
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
    MouseKeyRight.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_right.lock().unwrap().borrow().clone() {
            while MouseKeyRight.is_pressed() {
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
    MouseKeyUpperLeft.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_up_left.lock().unwrap().borrow().clone() {
            while MouseKeyUpperLeft.is_pressed() {
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
    MouseKeyUpperRight.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_up_right.lock().unwrap().borrow().clone() {
            while MouseKeyUpperRight.is_pressed() {
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
    MouseKeyLowerRight.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_down_right.lock().unwrap().borrow().clone() {
            while MouseKeyLowerRight.is_pressed() {
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
    MouseKeyLowerLeft.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_down_left.lock().unwrap().borrow().clone() {
            while MouseKeyLowerLeft.is_pressed() {
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

    MouseKeyClickToggle.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_click_toggle.lock().unwrap().borrow().clone() {
            //toggle whether left click is counted for num pad five
            let cur_value = **left_click_toggle.to_owned().lock().unwrap().borrow();
            left_click_toggle
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(!cur_value));
        }
    });
    //Numpad1Key.bind(|| {
    MouseKeyFastToggle.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_fast.lock().unwrap().borrow().clone() {
            //set fast speed
            let cur_value = **is_fast.clone().lock().unwrap().borrow();
            is_fast
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(!cur_value));
        }
    });
    //toggle is numlock on each time num lock key is pressed
    NumLockKey.bind(move || {
        let cur_value = **is_numlock_on.clone().lock().unwrap().borrow();
        is_numlock_on
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    });

    //Numpad5Key.bind(|| {
    MouseKeyMiddle.bind(move || {
        //if NumLockKey.is_toggled() {
        if *is_numlock_on_middle.lock().unwrap().borrow().clone() {
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
    //TODO: ensure this is moved to new signal
    NumpadPlusKey.bind(move || {
        //if NumLockKey.is_toggled() {
        if **is_numlock_on_plus.lock().unwrap().borrow() {
            //hold left click, released by another 5 left click
            if *left_click_hold.lock().unwrap().borrow().clone() {
                MouseButton::LeftButton.press();
            } else {
                //right
                MouseButton::RightButton.press();
            }
        }
    });

    //Numpad1Key.bind(move || {

    //TODO: hold mouse toggle
    //Numpad0Key.bind(|| {

    handle_input_events();
}
//TODO: feature for speed and acceleration etc. has to be a feature so it can be user defined easily
