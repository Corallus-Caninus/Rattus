use Rattus::data_logger::BindRecord;
use Rattus::data_logger::MouseAction;

//TODO: @DEPRECATED for windows api: fix arrow keys
use inputbot::{self, KeybdKey::*, *};

// use these to feed to Rat_Tunnel network and animate
// motions such as lines to track tunnel cursor teleport
use std::boxed::Box;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::{self};

use windows as Windows;
use windows::core::*;
use windows::Win32::Foundation::E_BOUNDS;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::UI::Input::Preview::Injection::*;
use windows_macros::implement;

use serde_derive::Deserialize;

// TODO: extract this to a crate or pr? this is a tmp patch and TODO on windows api but is useful as api needs iterables for many calls.
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
        panic!();
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

struct Sendjector(InputInjector);
unsafe impl Send for Sendjector {}

trait KeyPressed {
    fn is_pressed(&self) -> bool;
}
impl KeyPressed for i32 {
    fn is_pressed(&self) -> bool {
        unsafe {
            let is_pressed = GetKeyState(*self as i32);
            if is_pressed < 0 {
                true
            } else {
                false
            }
        }
    }
}

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
        x: i32,
        y: i32,
        history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
        let injector = injector.lock().unwrap().0.clone();

        //TODO: this should be in config toml
        if is_rat_on.load(Ordering::SeqCst) {
            //fallthrough move with medium speed
            let mut speed = medium_speed;
            //move with fast or slow speed
            if is_fast.load(Ordering::SeqCst) && is_slow.load(Ordering::SeqCst) {
                //move with fast speed
                // speed = (medium_speed - fast_speed) / 2;
                speed = (fast_speed - medium_speed) / 2;
            } else if is_fast.load(Ordering::SeqCst) {
                //move with slow speed
                speed = fast_speed;
            } else if is_slow.load(Ordering::SeqCst) {
                //move with slow speed
                speed = slow_speed;
            }

            //TODO: initialize touch input device and associate the id here for position tracking?
            let mouse_injection = InjectedInputMouseInfo::new().unwrap();
            mouse_injection.SetDeltaY(y * speed as i32).unwrap();
            mouse_injection.SetDeltaX(x * speed as i32).unwrap();
            let mouse_force_move = InjectedInputMouseOptions::MoveNoCoalesce; //coalesce other messages?
            mouse_injection.SetMouseOptions(mouse_force_move).unwrap();

            let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                Iterable(vec![mouse_injection]).into();
            let res = injector.InjectMouseInput(solution);

            //this seems to be a-okay, prefer here and not in polling loop for held keys etc.
            // sleep(Duration::from_millis(1 as u64));

            match res {
                Ok(_) => {}
                Err(e) => {
                    //ignore since windows api can fail quietly here
                }
            }
        } else if is_numlock_on.load(Ordering::SeqCst) {
            let mut arrow_speed = medium_arrow_speed;
            if is_fast.load(Ordering::SeqCst) && is_slow.load(Ordering::SeqCst) {
                arrow_speed = (medium_arrow_speed - fast_arrow_speed) / 2;
            } else if is_fast.load(Ordering::SeqCst) {
                arrow_speed = fast_arrow_speed;
            } else if is_slow.load(Ordering::SeqCst) {
                arrow_speed = slow_arrow_speed;
            }

            if mode_arrow_diagonal.is_none() {
                //click with inputbot
                mode_arrow.press();
                mode_arrow.release();
                sleep(Duration::from_micros(arrow_speed as u64));
            } else {
                mode_arrow_diagonal.unwrap().press();
                mode_arrow_diagonal.unwrap().release();
                sleep(Duration::from_micros(arrow_speed as u64));
            }
        } else {
            //press and release arrow with medium speed
            mode_keypad.press();
            sleep(Duration::from_micros(numpad_speed as u64));
            mode_keypad.release();
        }
    }
}

//NOTE: currently using sharpkeys program to map. would prefer programatic solution even if it
//      mutates the registry statically (with version warning/constraint for users) with generated sharpkey solution.
fn main() {
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

    //the history buffer of mouse clicks and current location
    let history = Arc::new(Mutex::new(RefCell::new(vec![])));
    //the stored procedures of the mouse where keys are 1-9 and values are
    //vectors of postitions and possible clicks
    // let mut robots = HashMap::new();

    let left_click_toggle = Arc::new(Mutex::new(RefCell::new(Box::new(true))));

    //create is_fast for up down left right and all diagonals
    let is_fast = Arc::new(AtomicBool::new(false));
    let is_slow = Arc::new(AtomicBool::new(false));

    // TODO: force this to sync with numlock on initialization or (preferably) keep led in sync otherwise
    let is_numlock_on = Arc::new(AtomicBool::new(true));
    let is_rat_on = Arc::new(AtomicBool::new(true));

    //TODO: these should be a builder that has defaults or at least defaults
    let injector = Arc::new(Mutex::new(Sendjector {
        0: InputInjector::TryCreate().unwrap(),
    }));
    injector
        .clone()
        .lock()
        .unwrap()
        .0
        .InitializeTouchInjection(InjectedInputVisualizationMode::Default)
        .unwrap();

    //BEGIN EVENT HANDLE
    // use winapi::{
    //     ctypes::*,
    //     shared::{minwindef::*, windef::*},
    //     um::winuser::*,
    // };
    // //import lazy and atomicptr
    // use once_cell::sync::Lazy;
    // //import MaybeUninit;
    // use core::ptr::null_mut;
    // use std::mem::MaybeUninit;
    // use std::sync::atomic::{AtomicPtr, Ordering};
    // static KEYBD_HHOOK: Lazy<AtomicPtr<HHOOK__>> = Lazy::new(AtomicPtr::default);
    // static MOUSE_HHOOK: Lazy<AtomicPtr<HHOOK__>> = Lazy::new(AtomicPtr::default);
    // pub use std::{collections::hash_map::HashMap, thread::spawn};

    // pub enum Bind {
    //     NormalBind(BindHandler),
    //     BlockBind(BlockBindHandler),
    //     BlockableBind(BlockableBindHandler),
    // }

    // pub type BindHandler = Arc<dyn Fn() + Send + Sync + 'static>;
    // pub type BlockBindHandler = Arc<dyn Fn() + Send + Sync + 'static>;
    // pub type BlockableBindHandler = Arc<dyn Fn() -> BlockInput + Send + Sync + 'static>;
    // pub type KeybdBindMap = HashMap<KeybdKeys, Bind>;
    // pub type MouseBindMap = HashMap<MouseButton, Bind>;

    // pub static KEYBD_BINDS: Lazy<Mutex<KeybdBindMap>> =
    //     Lazy::new(|| Mutex::new(KeybdBindMap::new()));
    // pub static MOUSE_BINDS: Lazy<Mutex<MouseBindMap>> =
    //     Lazy::new(|| Mutex::new(MouseBindMap::new()));

    // #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    // pub enum KeybdKeys {
    //     Numpad0Keys,
    //     Numpad1Keys,
    //     Numpad2Keys,
    //     Numpad3Keys,
    //     Numpad4Keys,
    //     Numpad5Keys,
    //     Numpad6Keys,
    //     Numpad7Keys,
    //     Numpad8Keys,
    //     Numpad9Keys,
    //     NumpadPlusKeys,
    //     NumpadDivKeys,
    //     NumpadDelKeys,
    //     NumpadEnterKeys,
    //     NumlockKeys,
    //     OtherKey(u64),
    // }
    // impl From<u64> for KeybdKeys {
    //     fn from(code: u64) -> KeybdKeys {
    //         use KeybdKeys::*;
    //         match code {
    //             // let Numpad9Keyi32 = 0x69;
    //             // let Numpad1Keyi32 = 0x61;
    //             // let Numpad3Keyi32 = 0x63;
    //             // let Numpad7Keyi32 = 0x67;
    //             // let Numpad8Keyi32 = 0x68;
    //             // let Numpad2Keyi32 = 0x62;
    //             // let Numpad4Keyi32 = 0x64;
    //             // let Numpad6Keyi32 = 0x66;
    //             0x60 => Numpad0Keys,
    //             0x61 => Numpad1Keys,
    //             // 0x73 => Numpad1Keys,
    //             // 0xC1 => Numpad1Keys,
    //             0x62 => Numpad2Keys,
    //             0x63 => Numpad3Keys,
    //             0x64 => Numpad4Keys,
    //             0x65 => Numpad5Keys,
    //             0x66 => Numpad6Keys,
    //             0x67 => Numpad7Keys,
    //             0x68 => Numpad8Keys,
    //             0x69 => Numpad9Keys,
    //             0x6E => NumpadPlusKeys,
    //             0x6F => NumpadDivKeys,
    //             0x6A => NumpadDelKeys,
    //             0x6D => NumpadEnterKeys,
    //             0x90 => NumlockKeys,
    //             _ => OtherKey(code),
    //         }
    //     }
    // }
    // impl From<KeybdKeys> for u64 {
    //     fn from(key: KeybdKeys) -> u64 {
    //         use KeybdKeys::*;
    //         match key {
    //             Numpad0Keys => 0x60,
    //             Numpad1Keys => 0x61,
    //             // Numpad1Keys => 0x73,
    //             // Numpad1Keys => 0xC1,
    //             Numpad2Keys => 0x62,
    //             Numpad3Keys => 0x63,
    //             Numpad4Keys => 0x64,
    //             Numpad5Keys => 0x65,
    //             Numpad6Keys => 0x66,
    //             Numpad7Keys => 0x67,
    //             Numpad8Keys => 0x68,
    //             Numpad9Keys => 0x69,
    //             NumpadPlusKeys => 0x6E,
    //             NumpadDivKeys => 0x6F,
    //             NumpadDelKeys => 0x6A,
    //             NumpadEnterKeys => 0x6D,
    //             NumlockKeys => 0x90,
    //             OtherKey(code) => code,
    //         }
    //     }
    // }
    // impl KeybdKeys {
    //     pub fn bind<F: Fn() + Send + Sync + 'static>(self, callback: F) {
    //         KEYBD_BINDS
    //             .lock()
    //             .unwrap()
    //             .insert(self, Bind::NormalBind(Arc::new(callback)));
    //     }

    //     pub fn block_bind<F: Fn() + Send + Sync + 'static>(self, callback: F) {
    //         KEYBD_BINDS
    //             .lock()
    //             .unwrap()
    //             .insert(self, Bind::BlockBind(Arc::new(callback)));
    //     }

    //     pub fn blockable_bind<F: Fn() -> BlockInput + Send + Sync + 'static>(self, callback: F) {
    //         KEYBD_BINDS
    //             .lock()
    //             .unwrap()
    //             .insert(self, Bind::BlockableBind(Arc::new(callback)));
    //     }

    //     pub fn unbind(self) {
    //         KEYBD_BINDS.lock().unwrap().remove(&self);
    //     }
    // }

    // pub fn handle_input_events() {
    //     // if !MOUSE_BINDS.lock().unwrap().is_empty() {
    //     //     set_hook(WH_MOUSE_LL, &*MOUSE_HHOOK, mouse_proc);
    //     // };
    //     // if !KEYBD_BINDS.lock().unwrap().is_empty() {
    //     set_hook(WH_KEYBOARD_LL, &*KEYBD_HHOOK, keybd_proc);
    //     // };
    //     let mut msg: MSG = unsafe { MaybeUninit::zeroed().assume_init() };
    //     unsafe { GetMessageW(&mut msg, 0 as HWND, 0, 0) };
    // }

    // unsafe extern "system" fn keybd_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    //     // if KEYBD_BINDS.lock().unwrap().is_empty() {
    //     //     unset_hook(&*KEYBD_HHOOK);
    //     // } else
    //     // println!("{:?}", code);
    //     std::thread::spawn(move || CallNextHookEx(null_mut(), code, w_param, l_param));
    //     let res = std::thread::spawn(move || {
    //         if w_param as u32 == WM_KEYDOWN {
    //             // if let Some(bind) = KEYBD_BINDS
    //             //     .lock()
    //             //     .unwrap()
    //             //     .get_mut(&KeybdKeys::from(u64::from(
    //             //         (*(l_param as *const KBDLLHOOKSTRUCT)).vkCode,
    //             let virtcode = (*(l_param as *const KBDLLHOOKSTRUCT)).vkCode;
    //             // println!("{:?}", virtcode);
    //             //if the virtcode is in KeybdKeys enum dont allow the key press (return 1)
    //             match KeybdKeys::from(virtcode as u64) {
    //                 KeybdKeys::Numpad0Keys
    //                 | KeybdKeys::Numpad1Keys
    //                 | KeybdKeys::Numpad2Keys
    //                 | KeybdKeys::Numpad3Keys
    //                 | KeybdKeys::Numpad4Keys
    //                 | KeybdKeys::Numpad5Keys
    //                 | KeybdKeys::Numpad6Keys
    //                 | KeybdKeys::Numpad7Keys
    //                 | KeybdKeys::Numpad8Keys
    //                 | KeybdKeys::Numpad9Keys
    //                 | KeybdKeys::NumpadPlusKeys
    //                 | KeybdKeys::NumpadDivKeys
    //                 | KeybdKeys::NumpadDelKeys
    //                 | KeybdKeys::NumlockKeys
    //                 | KeybdKeys::NumpadEnterKeys => {
    //                     // println!("got key: {:?}", KeybdKeys::from(virtcode as u64));
    //                     return 1;
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         return 0;
    //     });
    //     res.join().unwrap()

    //     // //return 1 if res returned 1 else CallNextHook
    //     // if res.join().unwrap() == 1 {
    //     //     return 1;
    //     // } else {
    //     //     0
    //     //     //     return CallNextHookEx(null_mut(), code, w_param, l_param);
    //     // }
    // }
    // fn set_hook(
    //     hook_id: i32,

    //     hook_ptr: &AtomicPtr<HHOOK__>,
    //     hook_proc: unsafe extern "system" fn(c_int, WPARAM, LPARAM) -> LRESULT,
    // ) {
    //     hook_ptr.store(
    //         unsafe { SetWindowsHookExW(hook_id, Some(hook_proc), 0 as HINSTANCE, 0) },
    //         // Ordering::Relaxed,
    //         Ordering::SeqCst,
    //     );
    // }

    // fn unset_hook(hook_ptr: &AtomicPtr<HHOOK__>) {
    //     if !hook_ptr.load(Ordering::Relaxed).is_null() {
    //         unsafe { UnhookWindowsHookEx(hook_ptr.load(Ordering::Relaxed)) };
    //         hook_ptr.store(null_mut(), Ordering::Relaxed);
    //     }
    // }
    //END EVENT HOOK

    //original bindings (not blocked so we modulate with sharpkeys)
    // let Numpad9Keyi32 = 0x69;
    // let Numpad1Keyi32 = 0x61;
    // let Numpad3Keyi32 = 0x63;
    // let Numpad7Keyi32 = 0x67;
    // let Numpad8Keyi32 = 0x68;
    // let Numpad2Keyi32 = 0x62;
    // let Numpad4Keyi32 = 0x64;
    // let Numpad6Keyi32 = 0x66;

    // NOTE: set with sharpkeys in windows registry

    // TODO: handle keypress event and block instead of mapping. less user configuration and more robust
    //TODO: manually edit these in registry and save in project to values not available in sharpkey
    //      (ensure there is a corresponding virtual key or create a new virtual key)
    let Numpad1Keyi32 = 0xC1;
    let Numpad2Keyi32 = 0xE9;
    let Numpad3Keyi32 = 0xFF;
    let Numpad4Keyi32 = 0xEE;
    let Numpad5Keyi32 = 0xF1;
    let Numpad6Keyi32 = 0xEA;
    let Numpad7Keyi32 = 0xF9;
    let Numpad8Keyi32 = 0xF5;
    let Numpad9Keyi32 = 0xF3;
    let Numpad0Keyi32 = 0xC2;
    let NumpadDelKeyi32 = 0x09;
    let NumpadDivKeyi32 = 0x2F;
    let NumpadAddKeyi32 = 0x87;
    let NumLockKeyi32 = 0x90;
    let NumpadEnterKeyi32 = 0x0C;
    // let Numpad1Keyi32 = u64::from(KeybdKeys::Numpad1Keys) as i32;
    // let Numpad2Keyi32 = u64::from(KeybdKeys::Numpad2Keys) as i32;
    // let Numpad3Keyi32 = u64::from(KeybdKeys::Numpad3Keys) as i32;
    // let Numpad4Keyi32 = u64::from(KeybdKeys::Numpad4Keys) as i32;
    // let Numpad5Keyi32 = u64::from(KeybdKeys::Numpad5Keys) as i32;
    // let Numpad6Keyi32 = u64::from(KeybdKeys::Numpad6Keys) as i32;
    // let Numpad7Keyi32 = u64::from(KeybdKeys::Numpad7Keys) as i32;
    // let Numpad8Keyi32 = u64::from(KeybdKeys::Numpad8Keys) as i32;
    // let Numpad9Keyi32 = u64::from(KeybdKeys::Numpad9Keys) as i32;
    // let Numpad0Keyi32 = u64::from(KeybdKeys::Numpad0Keys) as i32;
    // let NumpadDelKeyi32 = u64::from(KeybdKeys::NumpadDelKeys) as i32;
    // let NumpadDivKeyi32 = u64::from(KeybdKeys::NumpadDivKeys) as i32;
    // let NumpadAddKeyi32 = u64::from(KeybdKeys::NumpadPlusKeys) as i32;
    // let NumLockKeyi32 = u64::from(KeybdKeys::NumlockKeys) as i32;
    // let NumpadEnterKeyi32 = u64::from(KeybdKeys::NumpadEnterKeys) as i32;

    let mut click_timeout = SystemTime::now();

    // handle_input_events();
    //fork handle_input_events to a thread
    let handle_input_events_thread = std::thread::spawn(|| {
        handle_input_events();
    });

    // TODO: record mouse positions and clicks in history
    // TODO: scroll wheel
    loop {
        //scheduler frequency
        sleep(Duration::from_millis(10));
        //TODO: need to rec for brain
        // NOTE: this has one thread so sleep doesnt have to lock a mutex to stall for speed :)
        // MouseKeyUp.rat_move(
        if Numpad8Keyi32.is_pressed() {
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
                None,
                injector.clone(),
                0,
                -1,
                history.clone(),
            );
        }
        if Numpad2Keyi32.is_pressed() {
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
                None,
                injector.clone(),
                0,
                1,
                history.clone(),
            );
        }
        if Numpad4Keyi32.is_pressed() {
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
                None,
                injector.clone(),
                -1,
                0,
                history.clone(),
            );
        }
        if Numpad6Keyi32.is_pressed() {
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
                None,
                injector.clone(),
                1,
                0,
                history.clone(),
            );
        }
        if Numpad7Keyi32.is_pressed() {
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
                UpKey,
                Some(LeftKey),
                injector.clone(),
                -1,
                -1,
                history.clone(),
            );
        }
        if Numpad9Keyi32.is_pressed() {
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
                injector.clone(),
                1,
                -1,
                history.clone(),
            );
        }
        if Numpad3Keyi32.is_pressed() {
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
                injector.clone(),
                1,
                1,
                history.clone(),
            );
        }
        if Numpad1Keyi32.is_pressed() {
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
                injector.clone(),
                -1,
                1,
                history.clone(),
            );
        }
        //TODO: seperate thread loop for non movement keys
        //TODO: lock on the existing injector
        //TODO: cooldown for button press (different than click speed)
        //get the time passed since last click_timeout
        if Numpad5Keyi32.is_pressed()
            && click_timeout.elapsed().unwrap() > Duration::from_millis(click_speed as u64)
        {
            click_timeout = SystemTime::now();
            //toggle left click
            if is_rat_on.load(Ordering::SeqCst) {
                // let injector = InputInjector::TryCreate().unwrap();
                let injector = injector.clone().lock().unwrap().0.clone();
                let mouse_injection = InjectedInputMouseOptions::LeftDown;
                let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
                mouse_injection_solution
                    .SetMouseOptions(mouse_injection)
                    .unwrap();
                let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                    Iterable(vec![mouse_injection_solution]).into();
                let res = injector.InjectMouseInput(solution.clone());
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        //ignore
                    }
                }

                sleep(Duration::from_micros(click_speed as u64));

                let mouse_injection = InjectedInputMouseOptions::LeftUp;
                let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
                mouse_injection_solution
                    .SetMouseOptions(mouse_injection)
                    .unwrap();

                let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                    Iterable(vec![mouse_injection_solution]).into();
                let res = injector.InjectMouseInput(solution.clone());
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        //ignore
                    }
                }

                left_click_toggle
                    .to_owned()
                    .lock()
                    .unwrap()
                    .replace(Box::new(true));
            } else if !is_numlock_on.load(Ordering::SeqCst) {
                //TODO: inputbot is DEPRECATED
                Numpad5Key.press();
                sleep(Duration::from_micros(click_speed as u64));
                Numpad5Key.release();
            }
            // is_fast.clone(),
            // is_slow.clone(),
            // is_rat_on.clone(),
            // history.clone(),
            // );
        }
        if NumpadAddKeyi32.is_pressed()
            && click_timeout.elapsed().unwrap() > Duration::from_millis(click_speed as u64)
        {
            click_timeout = SystemTime::now();
            let injector = injector.clone().lock().unwrap().0.clone();

            let mouse_injection = InjectedInputMouseOptions::RightDown;
            let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
            mouse_injection_solution
                .SetMouseOptions(mouse_injection)
                .unwrap();
            let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                Iterable(vec![mouse_injection_solution]).into();
            let res = injector.InjectMouseInput(solution.clone());
            match res {
                Ok(_) => {}
                Err(e) => {
                    //ignore
                }
            }

            sleep(Duration::from_micros(click_speed as u64));

            let mouse_injection = InjectedInputMouseOptions::RightUp;
            let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
            mouse_injection_solution
                .SetMouseOptions(mouse_injection)
                .unwrap();

            let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                Iterable(vec![mouse_injection_solution]).into();
            let res = injector.InjectMouseInput(solution.clone());
            match res {
                Ok(_) => {}
                Err(e) => {
                    //ignore
                }
            }

            left_click_toggle
                .to_owned()
                .lock()
                .unwrap()
                .replace(Box::new(false));
        }

        if Numpad0Keyi32.is_pressed() {
            if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
                Numrow0Key.press();
            } else {
                let cur_value = is_fast.load(Ordering::SeqCst);
                is_fast.swap(true, Ordering::SeqCst);
            }
        } else {
            if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
                Numrow0Key.release();
            } else {
                let cur_value = is_fast.load(Ordering::SeqCst);
                is_fast.swap(false, Ordering::SeqCst);
            }
        }
        if NumpadEnterKeyi32.is_pressed() {
            if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
                EnterKey.press();
            } else {
                let cur_value = is_slow.load(Ordering::SeqCst);
                is_slow.swap(true, Ordering::SeqCst);
            }
        } else {
            if !is_numlock_on.load(Ordering::SeqCst) && !is_rat_on.load(Ordering::SeqCst) {
                EnterKey.release();
            } else {
                let cur_value = is_slow.load(Ordering::SeqCst);
                is_slow.swap(false, Ordering::SeqCst);
            }
        }

        if NumLockKeyi32.is_pressed() {
            let cur_value = is_numlock_on.load(Ordering::SeqCst);
            is_numlock_on.swap(!cur_value, Ordering::SeqCst);
        }

        if NumpadDivKeyi32.is_pressed() {
            let cur_value = is_rat_on.load(Ordering::SeqCst);
            is_rat_on.swap(!cur_value, Ordering::SeqCst);
            sleep(Duration::from_millis(click_speed as u64));
        }
        if NumpadDelKeyi32.is_pressed() {
            if *left_click_toggle.lock().unwrap().borrow().clone() {
                // let injector = InputInjector::TryCreate().unwrap();
                let injector = injector.clone().lock().unwrap().0.clone();
                let mouse_injection = InjectedInputMouseOptions::LeftDown;
                let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
                mouse_injection_solution
                    .SetMouseOptions(mouse_injection)
                    .unwrap();
                let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                    Iterable(vec![mouse_injection_solution]).into();
                let res = injector.InjectMouseInput(solution.clone());
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        //ignore
                    }
                }
            } else {
                //right
                // let injector = InputInjector::TryCreate().unwrap();
                let injector = injector.clone().lock().unwrap().0.clone();
                let mouse_injection = InjectedInputMouseOptions::RightDown;
                let mouse_injection_solution = InjectedInputMouseInfo::new().unwrap();
                mouse_injection_solution
                    .SetMouseOptions(mouse_injection)
                    .unwrap();
                let solution: Windows::Foundation::Collections::IIterable<InjectedInputMouseInfo> =
                    Iterable(vec![mouse_injection_solution]).into();
                let res = injector.InjectMouseInput(solution.clone());
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        //ignore
                    }
                }
            }
        }
    }
}
