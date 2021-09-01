use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
//use tensorflow::ops::Enter;
// use these to feed to Rat_Tunnel network and animate
// motions such as lines to track tunnel cursor teleport
use toml;
use x11::xlib::{XGetImage, XPutImage};
use x11::{xinput2, xlib}; //for config file
                          //import crate for delay
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;
use std::fs::File;
use std::{self, primitive};
use uinput;
use uinput::event::keyboard;
//import box
use enclose::enclose;
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};

//Config file Data Structure
#[derive(Deserialize)]
struct Config {
    click_speed: i32,
    fast_speed: i32,
    medium_speed: i32,
    slow_speed: i32,
    arrow_speed: i32,
}
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
        is_fast: Arc<Mutex<RefCell<Box<bool>>>>,
        is_slow: Arc<Mutex<RefCell<Box<bool>>>>,
        is_rat_on: Arc<Mutex<RefCell<Box<bool>>>>,
        is_numlock_on: Arc<Mutex<RefCell<Box<bool>>>>,
        fast_speed: u64,
        medium_speed: u64,
        slow_speed: u64,
        arrow_speed: u64,
        mode_keypad: KeybdKey,
        // mode_arrow: KeybdKey,
        mode_arrow: keyboard::Key,
        mode_alt_arrow: Option<keyboard::Key>,
        x: i32,
        y: i32,
    );
}
impl MoveRat for KeybdKey {
    fn move_rat(
        self,
        is_fast: Arc<Mutex<RefCell<Box<bool>>>>,
        is_slow: Arc<Mutex<RefCell<Box<bool>>>>,
        is_rat_on: Arc<Mutex<RefCell<Box<bool>>>>,
        is_numlock_on: Arc<Mutex<RefCell<Box<bool>>>>,
        fast_speed: u64,
        medium_speed: u64,
        slow_speed: u64,
        arrow_speed: u64,
        mode_keypad: KeybdKey,
        // mode_arrow: KeybdKey,
        mode_arrow: keyboard::Key,
        mode_alt_arrow: Option<keyboard::Key>,
        x: i32,
        y: i32,
    ) {
        //TODO bind with release instead of while pressed not all keys and keyboards support this
        self.bind(move || {
            while self.is_pressed() {
                if *is_rat_on.lock().unwrap().borrow().clone() {
                    let is_slow = *is_slow.lock().unwrap().borrow().clone();
                    let is_fast = *is_fast.lock().unwrap().borrow().clone();
                    //TODO: slow mode with mixing using plus key
                    //move up with fast or slow speed
                    if is_fast && is_slow {
                        //move up with fast speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(
                            (medium_speed - fast_speed) / 2 as u64,
                        ));
                    } else if is_fast {
                        //move up with slow speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(fast_speed as u64));
                    } else if is_slow {
                        //move up with slow speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(slow_speed as u64));
                    } else {
                        //move up with medium speed
                        MouseCursor::move_abs(x, y);
                        sleep(Duration::from_micros(medium_speed as u64));
                    }
                } else if *is_numlock_on.lock().unwrap().borrow().clone() {
                    //TODO: move all non mouse modes into a bind+release_bind paradigm
                    //TODO: consider not using uinput since stream buffer seems to have delay, what does xlib have native support for?
                    //TODO: consolidate this with inputbot in a way that is contributable
                    //TODO: arrow and numpad speed params
                    //TODO: hold ins/ent for n presses fast and slow mode based on speed
                    if mode_alt_arrow.is_none() {
                        KEYBD_DEVICE.lock().unwrap().click(&mode_arrow).unwrap();
                        KEYBD_DEVICE.lock().unwrap().synchronize().unwrap();
                        sleep(Duration::from_micros(arrow_speed as u64));
                    } else {
                        KEYBD_DEVICE
                            .lock()
                            .unwrap()
                            .click(&mode_alt_arrow.unwrap())
                            .unwrap();
                        KEYBD_DEVICE.lock().unwrap().click(&mode_arrow).unwrap();
                        KEYBD_DEVICE.lock().unwrap().synchronize().unwrap();
                        sleep(Duration::from_micros(arrow_speed as u64));
                    }
                } else {
                    //press and release arrow with medium speed
                    // mode_keypad.click(Duration::from_micros(medium_speed as u64));
                    mode_keypad.click(Duration::from_micros(arrow_speed as u64));
                }
            }
        });
    }
}


//TODO: use led settings for custom blink codes or other modal user feedback
fn main() {
    //TODO: this is to focus the virtual device
    AKey.release();
    sleep(Duration::from_millis(100));
    //open config file and read into toml struct
    let mut config_file = File::open("Rat_config.toml").unwrap();
    let mut config_string = String::new();
    config_file.read_to_string(&mut config_string).unwrap();
    let config: Config = toml::from_str(&config_string).unwrap();

    //TODO: configuration file: params are too large
    // let args = env::args().skip(1).collect::<Vec<String>>();
    // let mut args = args
    //     .into_iter()
    //     .map(|x| x.parse().unwrap())
    //     .collect::<Vec<i32>>();
    // let fast_speed = args.pop().unwrap();
    // let medium_speed = args.pop().unwrap();
    // let slow_speed = args.pop().unwrap();
    // let arrow_speed = args.pop().unwrap();
    // let click_speed = args.pop().unwrap();
    let fast_speed = config.fast_speed;
    let medium_speed = config.medium_speed;
    let slow_speed = config.slow_speed;
    let arrow_speed = config.arrow_speed;
    let click_speed = config.click_speed;

    //assert that fast is greater than medium etc with the message x must be faster than y
    assert!(
        fast_speed < medium_speed,
        "fast_speed must be greater than medium_speed"
    );
    assert!(
        medium_speed < slow_speed,
        "medium_speed must be greater than slow_speed"
    );

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
    let is_rat_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

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

    //TODO: these are a hack to prevent keypad from sending
    awaits.push(
        std::process::Command::new("xmodmap")
            .args(&["-e", r#"keycode 104 = 1000"#]) //TODO: wrong
            .spawn(),
    );
    //asterisk
    // awaits.push(
    //     std::process::Command::new("xmodmap")
    //         .args(&["-e", r#"keycode 63 = 914 914"#])
    //         .spawn(),
    // );
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

    //TODO: diagonals with two arrows
    MouseKeyUp.move_rat(
        // cloning here is weird but doesnt really matter since this is config
        // and i'll take what I can get from the borrow checker
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow8Key,
        // UpKey,
        keyboard::Key::Up,
        None,
        0,
        -1,
    );
    MouseKeyDown.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow2Key,
        // DownKey,
        keyboard::Key::Down,
        None,
        0,
        1,
    );
    MouseKeyLeft.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow4Key,
        // LeftKey,
        keyboard::Key::Left,
        None,
        -1,
        0,
    );
    MouseKeyRight.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow6Key,
        // RightKey,
        keyboard::Key::Right,
        None,
        1,
        0,
    );
    MouseKeyUpperLeft.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow7Key,
        //TODO: this should be up and left at the same time
        // UpKey,
        keyboard::Key::Up,
        Some(keyboard::Key::Left),
        -1,
        -1,
    );
    MouseKeyUpperRight.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow9Key,
        // UpKey,
        keyboard::Key::Up,
        Some(keyboard::Key::Right),
        1,
        -1,
    );
    MouseKeyLowerRight.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow3Key,
        // DownKey,
        keyboard::Key::Down,
        Some(keyboard::Key::Right),
        1,
        1,
    );
    MouseKeyLowerLeft.move_rat(
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        is_numlock_on.clone(),
        fast_speed as u64,
        medium_speed as u64,
        slow_speed as u64,
        arrow_speed as u64,
        Numrow1Key,
        // DownKey,
        keyboard::Key::Down,
        Some(keyboard::Key::Left),
        -1,
        1,
    );

    NumpadPlusKey.bind(enclose!((left_click_toggle=> right_click_toggle) move || {
            MouseButton::RightButton.press();
            sleep(Duration::from_micros(click_speed as u64));
            MouseButton::RightButton.release();

            right_click_toggle
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(false));
    }));

    MouseKeySlow.bind(enclose!((is_slow)move || {
        is_slow.to_owned().lock().unwrap().replace(Box::new(true));
    }));
    MouseKeySlow.release_bind(enclose!((is_slow) move||{
        is_slow.to_owned().lock().unwrap().replace(Box::new(false));
    }));
    MouseKeyFast.bind(enclose!((is_fast)move || {
        is_fast.to_owned().lock().unwrap().replace(Box::new(true));
    }));
    MouseKeyFast.release_bind(enclose!((is_fast) move||{
        is_fast.to_owned().lock().unwrap().replace(Box::new(false));
    }));

    //toggle is numlock on each time num lock key is pressed
    // MouseKeyActivate.bind(move || {
    MouseKeyNumlock.bind(enclose!((is_numlock_on)move || {
        let cur_value = **is_numlock_on.clone().lock().unwrap().borrow();
        is_numlock_on
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    }));
    MouseKeySlash.bind(enclose!((is_rat_on) move || {
        let cur_value = **is_rat_on.clone().lock().unwrap().borrow();
        is_rat_on
            .to_owned()
            .lock()
            .unwrap()
            .replace(Box::new(!cur_value));
    }));

    //Numpad5Key.bind(|| {
    MouseKeyMiddle.bind(
        enclose!((is_numlock_on, is_rat_on, left_click_toggle) move || {
                //toggle left click
                if **is_rat_on.clone().lock().unwrap().borrow() {
                    MouseButton::LeftButton.press();
                    sleep(Duration::from_micros(click_speed as u64));
                    MouseButton::LeftButton.release();
                    left_click_toggle
                        .to_owned()
                        .lock()
                        .unwrap()
                        .replace(Box::new(true));
                } else if !**is_numlock_on.clone().lock().unwrap().borrow() {
                    &KEYBD_DEVICE.lock().unwrap().press(&keyboard::Key::_5).unwrap();
                    &KEYBD_DEVICE.lock().unwrap().release(&keyboard::Key::_5).unwrap();
                }
        }),
    );

    //TODO: change these names in input
    MouseKeyClickToggle.bind(
        enclose!((is_numlock_on, is_rat_on, left_click_toggle=>left_click_hold) move ||{
                //hold left click. released by another left click
                if *left_click_hold.lock().unwrap().borrow().clone() {
                    MouseButton::LeftButton.press();
                } else {
                    //right
                    MouseButton::RightButton.press();
            }
        }),
    );

    handle_input_events();
}
