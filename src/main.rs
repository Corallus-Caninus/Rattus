use Rattus::data_logger::BindRecord;
use Rattus::data_logger::MouseAction;

use inputbot::{
  self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
};
// use these to feed to Rat_Tunnel network and animate
// motions such as lines to track tunnel cursor teleport
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::Read;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex, RwLock,
};
use std::thread::sleep;
use std::time::Duration;
use std::{self};

// use x11::xlib::{XGetImage, XPutImage, XStringToKeysym};
// use x11::{xinput2, xlib};
// use x11::xlib::XDrawLine;

// use uinput;
// use uinput::event::keyboard;

use enclose::enclose;
use serde_derive::{Deserialize, Serialize};

use toml;

//Config file Data Structure
#[derive(Deserialize)]
struct Config {
  click_speed: i32,
  fast_speed: i32,
  medium_speed: i32,
  slow_speed: i32,
  fast_arrow_speed: i32,
  medium_arrow_speed: i32,
  slow_arrow_speed: i32,
  numpad_speed: i32,
}

// TODO: also derive builder
// #[derive(Default)
// struct move_event{
//   //TODO: defaults
//   is_fast: Arc<AtomicBool>,
//   is_slow: Arc<AtomicBool>,
//   is_rat_on: Arc<AtomicBool>,
//   is_numlock_on: Arc<AtomicBool>,
//   fast_speed: u64,
//   medium_speed: u64,
//   slow_speed: u64,
//   fast_arrow_speed: u64,
//   medium_arrow_speed: u64,
//   slow_arrow_speed: u64,
//   numpad_speed: u64,
//   mode_keypad: KeybdKey,
//   mode_arrow: keyboard::Key,
//   mode_arrow_diagonal: Option<keyboard::Key>,
//   x: i32,
//   y: i32,
//   history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
// }
//TODO: derive builder(default)
trait RatMoves {
  fn rat_move(
    self,
    is_fast: Arc<AtomicBool>,
    is_slow: Arc<AtomicBool>,
    is_rat_on: Arc<AtomicBool>,
    is_numlock_on: Arc<AtomicBool>,
    fast_speed: u64,
    medium_speed: u64,
    slow_speed: u64,
    fast_arrow_speed: u64,
    medium_arrow_speed: u64,
    slow_arrow_speed: u64,
    numpad_speed: u64,
    mode_keypad: KeybdKey,
    mode_arrow: KeybdKey,
    // mode_arrow: keyboard::Key,
    // mode_arrow_diagonal: Option<keyboard::Key>,
    mode_arrow_diagonal: Option<KeybdKey>,
    x: i32,
    y: i32,
    history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
  );
}
//TODO: this should be builder, make a seperate struct rat_config and pass it here (the struct has a builder)
impl RatMoves for KeybdKey {
  fn rat_move(
    self,
    // is_fast: Arc<Mutex<RefCell<Box<bool>>>>,
    is_fast: Arc<AtomicBool>,
    is_slow: Arc<AtomicBool>,
    is_rat_on: Arc<AtomicBool>,
    is_numlock_on: Arc<AtomicBool>,
    fast_speed: u64,
    medium_speed: u64,
    slow_speed: u64,
    fast_arrow_speed: u64,
    medium_arrow_speed: u64,
    slow_arrow_speed: u64,
    numpad_speed: u64,
    mode_keypad: KeybdKey,
    // mode_arrow: KeybdKey,
    // mode_arrow: keyboard::Key,
    // mode_arrow_diagonal: Option<keyboard::Key>,
    mode_arrow: KeybdKey,
    mode_arrow_diagonal: Option<KeybdKey>,
    x: i32,
    y: i32,
    history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
  ) {
    //TODO bind with release instead of while pressed not all keys and keyboards support this
    self.bind_rec(
      enclose!((is_fast, is_slow, is_rat_on, is_numlock_on)move || {
      while self.is_pressed() {
      if is_rat_on.load(Ordering::SeqCst) {
      //fallthrough move with medium speed
      let mut speed = medium_speed;
      //move with fast or slow speed
      if is_fast.load(Ordering::SeqCst) && is_slow.load(Ordering::SeqCst) {
          //move with fast speed
          speed = (medium_speed - fast_speed) / 2;
      } else if is_fast.load(Ordering::SeqCst) {
          //move with slow speed
          speed = fast_speed;
      } else if is_slow.load(Ordering::SeqCst) {
          //move with slow speed
          speed = slow_speed;
      }

      MouseCursor::move_abs(x, y);
      sleep(Duration::from_micros(speed as u64));
      }else if is_numlock_on.load(Ordering::SeqCst) {
      //TODO: move all non mouse modes into a bind+release_bind paradigm
      //TODO: consider not using uinput since stream buffer seems to have delay,
      //      what does xlib have native support for?
      //TODO: consolidate this with inputbot in a way that is contributable
      //TODO: need to lock sleep since cb threads
      //      every keypress
      let mut arrow_speed = medium_arrow_speed;
      if is_fast.load(Ordering::SeqCst) && is_slow.load(Ordering::SeqCst) {
          arrow_speed = (medium_arrow_speed - fast_arrow_speed) / 2;
      } else if is_fast.load(Ordering::SeqCst) {
          arrow_speed = fast_arrow_speed;
      } else if is_slow.load(Ordering::SeqCst) {
          arrow_speed = slow_arrow_speed;
      }

      if mode_arrow_diagonal.is_none() {
          // KEYBD_DEVICE.lock().unwrap().click(&mode_arrow).unwrap();
          // KEYBD_DEVICE.lock().unwrap().synchronize().unwrap();
          //click with inputbot
          mode_arrow.press();
          mode_arrow.release();
          sleep(Duration::from_micros(arrow_speed as u64));
      } else {
          // KEYBD_DEVICE
          //     .lock()
          //     .unwrap()
          //     .click(&mode_arrow_diagonal.unwrap())
          //     .unwrap();
          // KEYBD_DEVICE.lock().unwrap().click(&mode_arrow).unwrap();
          // KEYBD_DEVICE.lock().unwrap().synchronize().unwrap();
          mode_arrow_diagonal.unwrap().press();
          mode_arrow_diagonal.unwrap().release();
          sleep(Duration::from_micros(arrow_speed as u64));
      }
      } else {
      //press and release arrow with medium speed
      //TODO: add the rest of keypad in such aas + etc
      //TODO: numpad speed params
      mode_keypad.press();
      sleep(Duration::from_micros(numpad_speed as u64));
      mode_keypad.release();
      }
      }
      }),
      is_fast.clone(),
      is_slow.clone(),
      is_rat_on.clone(),
      history,
    );
  }
}

//TODO: use led settings for custom blink codes or other modal user feedback
fn main() {
  //TODO: this is to focus the virtual device and needs to be deprecated
  AKey.release();
  sleep(Duration::from_millis(100));

  //open config file and read toml into config struct
  let mut config_file = File::open("Rat_config.toml").unwrap();
  let mut config_string = String::new();
  config_file.read_to_string(&mut config_string).unwrap();
  let config: Config = toml::from_str(&config_string).unwrap(); //test

  let fast_speed = config.fast_speed;
  let medium_speed = config.medium_speed;
  let slow_speed = config.slow_speed;
  let fast_arrow_speed = config.fast_arrow_speed;
  let medium_arrow_speed = config.medium_arrow_speed;
  let slow_arrow_speed = config.slow_arrow_speed;
  let numpad_speed = config.numpad_speed;
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
  let history = Arc::new(Mutex::new(RefCell::new(vec![])));
  //the stored procedures of the mouse where keys are 1-9 and values are
  //vectors of postitions and possible clicks
  // let mut robots = HashMap::new();

  let left_click_toggle = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

  //create is_fast for up down left right and all diagonals
  // let is_fast = Arc::new(Mutex::new(Cell::new(false)));
  // let is_slow = Arc::new(Mutex::new(Cell::new(false)));
  let is_fast = Arc::new(AtomicBool::new(false));
  let is_slow = Arc::new(AtomicBool::new(false));

  // TODO: force this to sync with numlock on initialization or (preferably) keep led in sync otherwise
  // let is_numlock_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
  // let is_rat_on = Arc::new(Mutex::new(RefCell::new(Box::new(true))));
  let is_numlock_on = Arc::new(AtomicBool::new(true));
  let is_rat_on = Arc::new(AtomicBool::new(true));

  //  Num_Lock can't keep up so we need to write our own stateful modes using different toggle keys
  // let mut awaits = vec![];

  //TODO: these should be a builder that has defaults or at least defaults
  // MouseKeyUp.rat_move(
  Numpad8Key.rat_move(
    // cloning here is weird but doesnt really matter since this is config
    // and i'll take what I can get from the borrow checker
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow8Key,
    UpKey,
    // keyboard::Key::Up,
    None,
    0,
    -1,
    history.clone(),
  );
  // MouseKeyDown.rat_move(
  Numpad2Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow2Key,
    DownKey,
    // keyboard::Key::Down,
    None,
    0,
    1,
    history.clone(),
  );
  // MouseKeyLeft.rat_move(
  Numpad4Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow4Key,
    LeftKey,
    // keyboard::Key::Left,
    None,
    -1,
    0,
    history.clone(),
  );
  // MouseKeyRight.rat_move(
  Numpad6Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow6Key,
    RightKey,
    // keyboard::Key::Right,
    None,
    1,
    0,
    history.clone(),
  );
  // MouseKeyUpperLeft.rat_move(
  Numpad7Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow7Key,
    //TODO: this should be up and left at the same time
    // keyboard::Key::Up,
    // Some(keyboard::Key::Left),
    UpKey,
    Some(LeftKey),
    -1,
    -1,
    history.clone(),
  );
  // MouseKeyUpperRight.rat_move(
  Numrow9Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow9Key,
    UpKey,
    Some(RightKey),
    // keyboard::Key::Up,
    // Some(keyboard::Key::Right),
    1,
    -1,
    history.clone(),
  );
  // MouseKeyLowerRight.rat_move(
  Numrow3Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow3Key,
    DownKey,
    Some(RightKey),
    // keyboard::Key::Down,
    // Some(keyboard::Key::Right),
    1,
    1,
    history.clone(),
  );
  // MouseKeyLowerLeft.rat_move(
  Numpad1Key.rat_move(
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    is_numlock_on.clone(),
    fast_speed as u64,
    medium_speed as u64,
    slow_speed as u64,
    fast_arrow_speed as u64,
    medium_arrow_speed as u64,
    slow_arrow_speed as u64,
    numpad_speed as u64,
    Numrow1Key,
    DownKey,
    Some(LeftKey),
    // keyboard::Key::Down,
    // Some(keyboard::Key::Left),
    -1,
    1,
    history.clone(),
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

  Numpad0Key.bind(enclose!((is_slow, is_numlock_on, is_rat_on)move || {
  if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
  EnterKey.press();
  }else{
  is_slow.swap(true, Ordering::SeqCst);
  }
  }));
  Numpad0Key.release_bind(enclose!((is_slow, is_numlock_on,is_rat_on) move||{
  if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
  EnterKey.release();
  }else{
  is_slow.swap(false, Ordering::SeqCst);
  }
  }));
  NumpadPlusKey.bind(enclose!((is_fast, is_numlock_on, is_rat_on)move || {
  if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
  Numrow0Key.press();
  }else{
  is_fast.swap(true, Ordering::SeqCst);
  }
  }));
  Numpad0Key.release_bind(enclose!((is_fast, is_numlock_on, is_rat_on) move||{
  if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
  Numrow0Key.release();
  }else{
  is_fast.swap(false, Ordering::SeqCst);
  }
  }));

  //toggle is numlock on each time num lock key is pressed
  // MouseKeyActivate.bind(move || {
  NumLockKey.bind(enclose!((is_numlock_on)move || {
  let cur_value = is_numlock_on.load(Ordering::SeqCst);
  is_numlock_on.swap(!cur_value, Ordering::SeqCst);
  }));
  //TODO: would rather allow slash to operate with rapid numlock or
  //      something more appropriate for people with disabilities
  //      (hold for 3 or n seconds?)
  NumpadDivKey.bind(enclose!((is_rat_on) move || {
  let cur_value = is_rat_on.load(Ordering::SeqCst);
  is_rat_on.swap(!cur_value, Ordering::SeqCst);
  }));

  // MouseKeyMiddle.bind_rec(
  Numpad5Key.bind_rec(
    enclose!((is_numlock_on, is_rat_on, left_click_toggle) move || {
    //toggle left click
    if is_rat_on.load(Ordering::SeqCst) {
    MouseButton::LeftButton.press();
    sleep(Duration::from_micros(click_speed as u64));
    MouseButton::LeftButton.release();
    left_click_toggle
    .to_owned()
    .lock()
    .unwrap()
    .replace(Box::new(true));
    } else if !is_numlock_on.load(Ordering::SeqCst) {
    // &KEYBD_DEVICE.lock().unwrap().press(&keyboard::Key::_5).unwrap();
    // &KEYBD_DEVICE.lock().unwrap().release(&keyboard::Key::_5).unwrap();
    Numpad5Key.press();
    Numpad5Key.release();
    }
    }),
    is_fast.clone(),
    is_slow.clone(),
    is_rat_on.clone(),
    history,
  );

  //TODO: change these names in input
  //TODO: only toggle this in rat mode
  NumpadDelKey.bind(enclose!((left_click_toggle=>left_click_hold) move ||{
  //hold left click. released by another left click
  if *left_click_hold.lock().unwrap().borrow().clone() {
  MouseButton::LeftButton.press();
  } else {
  //right
  MouseButton::RightButton.press();
  }
  }));

  handle_input_events();
}
