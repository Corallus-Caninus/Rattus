use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
// use these to feed to Rat_Tunnel network and animate
// motions such as lines to track tunnel cursor teleports
use x11::xlib::{XGetImage, XPutImage};
use x11::{xinput2, xlib};
//import crate for delay
use std;
use std::collections::HashMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;
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
        fast_speed: u64,
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
        fast_speed: u64,
        slow_speed: u64,
        mode_keypad: KeybdKey,
        mode_arrow: KeybdKey,
        x: i32,
        y: i32,
    ) {
        self.bind(enclose!((is_numlock_on, is_fast) move || {
            if *is_numlock_on.lock().unwrap().borrow().clone() {
                while self.is_pressed() {
                    //move up with fast or slow speed
                    if *is_fast.lock().unwrap().borrow().clone() {
                        //move up with fast speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(fast_speed as u64));
                    } else {
                        //move up with slow speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(slow_speed as u64));
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
    //TODO: NKRO locks this on mutex: if two or more buttons are pressed at the same time as is_fast is toggled
    let is_fast = Arc::new(Mutex::new(RefCell::new(Box::new(false))));

    // TODO: force this to sync with numlock on initialization
    let is_numlock_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

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

    //TODO: restore default on close if no better solution found (not a priority)
    //  Num_Lock can't keep up so we need to write our own toggle using fast rust code and then pass through the
    //  num pad arrow keys and numbers respectively
    //  start by removing kp instructions here
    let mut awaits = vec![];
    //TODO: this should be a for loop
    //KP_Home
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 79 = 900"#])
            .spawn(), //.output(),
    );
    //KP_Up
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 80 = 901"#])
            .spawn(),
    );
    //KP_Prior
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 81 = 902"#])
            .spawn(),
    );
    //KP_Subtract
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 82 = 903"#])
            .spawn(),
    );
    //KP_Left
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 83 = 904"#])
            .spawn(),
    );
    //KP_Begin
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 84 = 905"#])
            .spawn(),
    );
    //KP_Right
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 85 = 906"#])
            .spawn(),
    );
    //KP_Add
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 86 = 907"#])
            .spawn(),
    );
    //KP_End
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 87 = 908"#])
            .spawn(),
    );
    //KP_Down
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 88 = 909"#])
            .spawn(),
    );
    //KP_Next
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 89 = 910"#])
            .spawn(),
    );
    //KP_Insert
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 90 = 911"#])
            .spawn(),
    );
    //KP_Delete
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 91 = 912 912"#])
            .spawn(),
    );
    //TODO: Enter key
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 104 = 913 913"#])
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
            .args(&["-e", r#"keycode 106 = 915 915"#])
            .spawn(),
    );
    //TODO: not implemented
    //also remap numlock since NKRO numpads dont arrive in order at usb
    //hub causing entries to not have numlock signal prepended
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 77=916 916"#])
            .spawn(),
    );
    awaits.into_iter().for_each(|x| {
        x.unwrap();
    });

    MouseKeyUp.move_rat(
        // cloning here is weird but doesnt really matter since this is config
        // and i'll take what I can get from the borrow checker
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
        slow_speed as u64,
        Numpad8Key,
        UpKey,
        0,
        -1,
    );
    MouseKeyDown.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
        slow_speed as u64,
        Numpad2Key,
        DownKey,
        0,
        1,
    );
    MouseKeyLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
        slow_speed as u64,
        Numpad4Key,
        LeftKey,
        -1,
        0,
    );
    MouseKeyRight.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
        slow_speed as u64,
        Numpad6Key,
        RightKey,
        1,
        0,
    );
    MouseKeyUpperLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
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
        fast_speed as u64,
        slow_speed as u64,
        Numpad9Key,
        UpKey,
        1,
        -1,
    );
    MouseKeyLowerRight.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
        slow_speed as u64,
        Numpad3Key,
        DownKey,
        1,
        1,
    );
    MouseKeyLowerLeft.move_rat(
        is_numlock_on.clone(),
        is_fast.clone(),
        fast_speed as u64,
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
    MouseKeyFast.bind(enclose!((is_numlock_on=>is_numlock_on_fast)move || {
        if *is_numlock_on_fast.lock().unwrap().borrow().clone() {
            //set fast speed
            is_fast.to_owned().lock().unwrap().replace(Box::new(true));
            // fast is not modal for ergonomics.
            while MouseKeyFast.is_pressed() {
                sleep(Duration::from_micros(slow_speed as u64));
                continue;
            }
            is_fast.to_owned().lock().unwrap().replace(Box::new(false));
        }
    }));
    //toggle is numlock on each time num lock key is pressed
    // MouseKeyActivate.bind(move || {
    NumLockKey.bind(enclose!((is_numlock_on) move || {
        let cur_value = **is_numlock_on.clone().lock().unwrap().borrow();
        is_numlock_on
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    }));

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