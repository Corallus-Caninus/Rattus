#![windows_subsystem = "windows"]

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

use windows::UI::Input::Preview::Injection::*;
// use windows::runtime::{IInspectable, IUnknown, IntoParam};
// use windows::Foundation::Collections::{IIterable, IVector, VectorIterator};
use windows as Windows;
use windows_sys::Foundation::Collections::IIterable;
// use windows::runtime::*;
use windows::core::*;
use windows::Win32::Foundation::E_BOUNDS;
use windows_macros::implement;

use enclose::enclose;
use serde_derive::{Deserialize, Serialize};

//TODO: fix formatting indentation.

// TODO: extract this to a crate or pr?
#[implement(
    Windows::Foundation::Collections::IIterator<T>,
)]
struct Iterator<T>
where
    T: RuntimeType + 'static,
{
    owner: Windows::Foundation::Collections::IIterable<T>,
    current: usize,
}
#[allow(non_snake_case)]
impl<T: RuntimeType + 'static> Iterator<T> {
    fn Current(&self) -> Result<T> {
        let owner = unsafe { Iterable::to_impl(&self.owner) };
        if owner.0.len() > self.current {
            Ok(owner.0[self.current].clone())
        } else {
            Err(Error::new(E_BOUNDS, "".into()))
        }
    }

    fn HasCurrent(&self) -> Result<bool> {
        let owner = unsafe { Iterable::to_impl(&self.owner) };
        Ok(owner.0.len() > self.current)
    }

    fn MoveNext(&mut self) -> Result<bool> {
        let owner = unsafe { Iterable::to_impl(&self.owner) };
        self.current += 1;
        Ok(owner.0.len() > self.current)
    }

    fn GetMany(&self, _items: &mut [<T as DefaultType>::DefaultType]) -> Result<u32> {
        panic!(); // TODO: arrays still need some work.
    }
}

#[implement(
      Windows::Foundation::Collections::IIterable<T>,
)]
struct Iterable<T>(Vec<T>)
where
    T: RuntimeType + 'static;

#[allow(non_snake_case)]
impl<T: RuntimeType + 'static> Iterable<T> {
    fn First(&mut self) -> Result<Windows::Foundation::Collections::IIterator<T>> {
        Ok(Iterator::<T> {
            owner: self.into(),
            current: 0,
        }
        .into())
    }
}

//impl unsafe Send for Injector which is NonNull<c_void>
struct Sendjector(InputInjector);
unsafe impl Send for Sendjector {}

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
//   TODO: keybdkey instead of extension trait inheratence
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
        mode_arrow_diagonal: Option<KeybdKey>,
        injector: Arc<Mutex<Sendjector>>,
        // inject_mouse_input: fn(Arc<Mutex<RefCell<Box<InputInjector>>>>, InjectedInputMouseInfo),
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
        mode_arrow: KeybdKey,
        mode_arrow_diagonal: Option<KeybdKey>,
        injector: Arc<Mutex<Sendjector>>,
        //inject_mouse_input is a dyn function that takes injector as self
        // inject_mouse_input: fn(Arc<Mutex<RefCell<Box<InputInjector>>>>, InjectedInputMouseInfo),
        x: i32,
        y: i32,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
        //TODO bind with is_pressed instead of while pressed not all keys and keyboards support this
        self.bind_rec(
        enclose!((is_fast, is_slow, is_rat_on, is_numlock_on, injector)move || {
            // let injector = InputInjector::TryCreate().unwrap();
            let injector = injector.lock().unwrap().0.clone();

            // while self.is_pressed() {
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

            for _ in 0..speed {
                let mouse_injection = InjectedInputMouseInfo::new().unwrap();
                mouse_injection.SetDeltaY(y as i32).unwrap();
                mouse_injection.SetDeltaX(x as i32).unwrap();
                let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> = Iterable(vec![mouse_injection]).into();
                injector.InjectMouseInput(solution.clone()).unwrap();
            }
            //now inject 0 or x and y to flush the buffer
            // for _ in 0..5{
            //     // let injector = InputInjector::TryCreate().unwrap();
            //     let mouse_injection = InjectedInputMouseInfo::new().unwrap();
            //     mouse_injection.SetDeltaY(1).unwrap();
            //     mouse_injection.SetDeltaX(0).unwrap();
            //     let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> = Iterable(vec![mouse_injection]).into();
            //     injector.lock().unwrap().0.InjectMouseInput(solution.clone()).unwrap();
            //     let mouse_injection = InjectedInputMouseInfo::new().unwrap();
            //     mouse_injection.SetDeltaY(-1).unwrap();
            //     mouse_injection.SetDeltaX(0).unwrap();
            //     let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> = Iterable(vec![mouse_injection]).into();
            //     injector.lock().unwrap().0.InjectMouseInput(solution.clone()).unwrap();
            // }

            // injector.InjectMouseInput(solution.clone()).unwrap();
            println!("{} {}", x, y);
            // sleep(Duration::from_micros(speed as u64));

            }else if is_numlock_on.load(Ordering::SeqCst) {

            //TODO: move all non mouse modes into a bind+release_bind paradigm
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
        // }
            // for _ in 0..100 {
            //     let injector = InputInjector::TryCreate().unwrap();
            //     let mouse_injection = InjectedInputMouseInfo::new().unwrap();
            //     mouse_injection.SetDeltaY(0 as i32).unwrap();
            //     mouse_injection.SetDeltaX(0 as i32).unwrap();
            //     let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
            //         Iterable(vec![mouse_injection]).into();
            //     injector.InjectMouseInput(solution.clone()).unwrap();
            // }
        }),
        is_fast.clone(),
        is_slow.clone(),
        is_rat_on.clone(),
        history,
        );
        //flush the keyboard buffer
        //TODO: this is a workaround the windows api raise issue
        self.release_bind(|| {});
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

    // we need to rewrite the following code in rust to block NumpadKey presses
    // #include <Windows.h>
    // #include <stdio.h>

    // static LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam)
    // {
    //     if ((wParam == WM_KEYDOWN || wParam == WM_KEYUP) && lParam != NULL)
    //     {
    //         if (((LPKBDLLHOOKSTRUCT)lParam)->vkCode == VK_OEM_MINUS)
    //         {
    //             return 1;
    //         }
    //     }

    //     return CallNextHookEx(NULL, nCode, wParam, lParam);
    // }

    // int main()
    // {
    //     HHOOK hHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, NULL, 0);
    //     MessageBoxW(NULL, L"hooking", L"", MB_ICONEXCLAMATION | MB_SYSTEMMODAL);
    //     UnhookWindowsHookEx(hHook);
    //     return 0;
    // }

    //TODO: these should be a builder that has defaults or at least defaults
    let injector = Arc::new(Mutex::new(Sendjector{0: InputInjector::TryCreate().unwrap()}));

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
        injector.clone(),
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
        injector.clone(),
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
        injector.clone(),
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
        injector.clone(),
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
        injector.clone(),
        -1,
        -1,
        history.clone(),
    );
    // MouseKeyUpperRight.rat_move(
    Numpad9Key.rat_move(
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
        injector.clone(),
        1,
        -1,
        history.clone(),
    );
    // MouseKeyLowerRight.rat_move(
    Numpad3Key.rat_move(
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
        injector.clone(),
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
        injector.clone(),
        -1,
        1,
        history.clone(),
    );

    // TODO: fix this formatting
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

    //TODO: for now speed modes are sticky because windows is possibly slow or I am missing something
    NumpadEnterKey.bind(enclose!((is_slow, is_numlock_on, is_rat_on)move || {
        if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
            EnterKey.press();
        }else{
            // is_slow.swap(true, Ordering::SeqCst);
            //reverse the cunnent value
            let cur_value = is_slow.load(Ordering::SeqCst);
            is_slow.swap(!cur_value, Ordering::SeqCst);
        }
    }));
    // Numpad0Key.release_bind(enclose!((is_slow, is_numlock_on,is_rat_on) move||{
    // if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
    // EnterKey.release();
    // }else{
    // is_slow.swap(false, Ordering::SeqCst);
    // }
    // }));
    Numpad0Key.block_bind(enclose!((is_fast, is_numlock_on, is_rat_on)move || {
        if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
            Numrow0Key.press();
        }else{
            // is_fast.swap(true, Ordering::SeqCst);
            //reverse the current value
            let cur_value = is_fast.load(Ordering::SeqCst);
            is_fast.swap(!cur_value, Ordering::SeqCst);
        }
    }));
    // Numpad0Key.release_bind(enclose!((is_fast, is_numlock_on, is_rat_on) move||{
    //     if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
    //         Numrow0Key.release();
    //     }else{
    //         is_fast.swap(false, Ordering::SeqCst);
    //     }
    // }));

    //toggle is numlock on each time num lock key is pressed
    // MouseKeyActivate.bind(move || {
    NumLockKey.bind(enclose!((is_numlock_on)move || {
        let cur_value = is_numlock_on.load(Ordering::SeqCst);
        is_numlock_on.swap(!cur_value, Ordering::SeqCst);
    }));
    //TODO: would rather allow slash to operate with rapid numlock or
    //      something more appropriate for people with disabilities
    //      (hold for 3 or n seconds?)
    NumpadDivKey.block_bind(enclose!((is_rat_on) move || {
        let cur_value = is_rat_on.load(Ordering::SeqCst);
        is_rat_on.swap(!cur_value, Ordering::SeqCst);
    }));

    // MouseKeyMiddle.bind_rec(
    //TODO: use uwp touch instead of mouse
    Numpad5Key.bind_rec(
        enclose!((is_numlock_on, is_rat_on, left_click_toggle) move || {
        //toggle left click
        if is_rat_on.load(Ordering::SeqCst) {
            //@DEPRECATED
            // MouseButton::LeftButton.press();
            // sleep(Duration::from_micros(click_speed as u64));
            // MouseButton::LeftButton.release();
            let injector = InputInjector::TryCreate().unwrap();
            let mouse_injection = InjectedInputMouseOptions::LeftDown;
            let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
            mouse_injection_solution.SetMouseOptions(mouse_injection).unwrap();
            let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> = Iterable(vec![mouse_injection_solution]).into();
            injector.InjectMouseInput(solution.clone()).unwrap();

            sleep(Duration::from_micros(click_speed as u64));

            let mouse_injection = InjectedInputMouseOptions::LeftUp;
            let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();  
            mouse_injection_solution.SetMouseOptions(mouse_injection).unwrap();

            let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> = Iterable(vec![mouse_injection_solution]).into();
            injector.InjectMouseInput(solution.clone()).unwrap();

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
    NumpadDelKey.block_bind(enclose!((left_click_toggle=>left_click_hold) move ||{
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
