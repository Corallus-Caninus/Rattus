pub mod rat_brain;

pub mod data_logger {
  //! data logging data structures and utilities for basic data preperation from Rattus.
  use enclose::enclose;
  use inputbot::{
    self, handle_input_events, KeySequence, KeybdKey::*, MouseButton::*, MouseCursor, *,
  };
  use serde::Serialize;
  use serde_derive::{Deserialize, Serialize};
  //import refcell
  use std::cell::{Cell, RefCell};
  //import arc
  use std::sync::Arc;
  //import mutex
  use std::sync::Mutex;
  //import thread
  use std::thread::spawn;
  //import AtomicBool
  use std::sync::atomic::{AtomicBool, Ordering};
  //import file for writting MouseActions
  use std::fs::File;
  use std::fs::OpenOptions;
  //import Error for windows
  use std::io::Error;
  use std::os::windows::io::*;
  use std::io::Write;
  use std::io::{self, BufRead};

  ///Mouse action for data collection
  ///used in machine learning algorithms.
  #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
  pub struct MouseAction {
    //Whatever the precision of the monitor is in x/y pixels.
    pub location: (i32, i32),
    pub is_clicked: bool,
    //Whether the fast or slow mode is used.
    pub is_fast: bool,
    pub is_slow: bool,
    //switch between original numpad mode and Rattus.
    pub is_rat_on: bool,
  }
  ///Sub-struct of the MouseAction struct
  ///that indicates location and modes but not click
  ///used in machine learning algorithms so the types
  ///default to f32.
  #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
  pub struct MouseActionLocation {
    pub location: (f32, f32),
    pub is_fast: f32,
    pub is_slow: f32,
    pub is_rat_on: f32,
  }

  //TODO: need to start using lifetimes
  ///extension trait to record each keyboard event with a proxy inheritence
  pub trait BindRecord {
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
      self,
      f: F,
      is_fast: Arc<AtomicBool>,
      is_slow: Arc<AtomicBool>,
      is_rat_on: Arc<AtomicBool>,
      history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    );
    fn bind_release_rec<F: Fn() + Send + Sync + 'static>(
      self,
      f: F,
      is_fast: Arc<AtomicBool>,
      is_slow: Arc<AtomicBool>,
      is_rat_on: Arc<AtomicBool>,
      history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    );
  }
  impl BindRecord for KeybdKey {
    ///binds a function to a key press and also records
    ///keypress as a mouse action in the history buffer
    fn bind_rec<F: Fn() + Send + Sync + 'static>(
      self,
      f: F,
      is_fast: Arc<AtomicBool>,
      is_slow: Arc<AtomicBool>,
      is_rat_on: Arc<AtomicBool>,
      history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
      self.bind(
        enclose!((self=>cur_key, is_fast, is_slow,is_rat_on, history)move || {
            f();

            spawn(enclose!((cur_key, is_fast, is_slow,is_rat_on, history)move || {
            let mut is_clicked = false;
            if u64::from(cur_key.to_owned()) == u64::from(KeybdKey::Numpad5Key) {
                //this was a click
                is_clicked = true;
            }
            let position = inputbot::MouseCursor::pos();

            //now get cursor location
            let cur_action = MouseAction {
                location: position,
                is_clicked: is_clicked,
                is_fast: is_fast.load(Ordering::SeqCst),
                is_slow: is_slow.load(Ordering::SeqCst),
                is_rat_on: is_rat_on.load(Ordering::SeqCst),
            };
            history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
            //TODO: write less and put a cap on in memory size at some point
            let mut file = OpenOptions::new().write(true).append(true).open("rat_nest").unwrap();
            // file.write_all(format!("{},{},{},{},{},{} \n",
            //     cur_action.location.0,
            //     cur_action.location.1,
            //     cur_action.is_clicked as u8,
            //     cur_action.is_fast as u8,
            //     cur_action.is_slow as u8,
            //     cur_action.is_rat_on as u8)
            //     .as_bytes()).unwrap();
            // same as above but for windows
            file.write_all(format!("{},{},{},{},{},{} \n",
                cur_action.location.0,
                cur_action.location.1,
                cur_action.is_clicked as u8,
                cur_action.is_fast as u8,
                cur_action.is_slow as u8,
                cur_action.is_rat_on as u8)
                .as_bytes()).unwrap();
            }));
        }),
      );
    }

    ///bind a function to be called when the key is released
    ///and also store the action in the history buffer
    fn bind_release_rec<F: Fn() + Send + Sync + 'static>(
      self,
      f: F,
      is_fast: Arc<AtomicBool>,
      is_slow: Arc<AtomicBool>,
      is_rat_on: Arc<AtomicBool>,
      history: Arc<Mutex<RefCell<Vec<MouseAction>>>>,
    ) {
      // self.bind(enclose!((history) move || {
      self.bind(enclose!((is_fast, is_slow,is_rat_on, history)move || {
      f();

      spawn(enclose!((is_fast, is_slow, is_rat_on, history) move || {
              let position = inputbot::MouseCursor::pos();

              //now get cursor location
              let cur_action = MouseAction {
                  location: position,
                  is_clicked: false,
                  is_fast: is_fast.load(Ordering::SeqCst),
                  is_slow: is_slow.load(Ordering::SeqCst),
                  is_rat_on: is_rat_on.load(Ordering::SeqCst),
              };
              history.to_owned().lock().unwrap().borrow_mut().push(cur_action);
              //close scope
          }));
      }));
    }
  }

  use rand::prelude::*;
  use std::iter::FromIterator;
  use std::iter::Iterator;
  ///Data for trainning a machine learning algorithm and data for
  ///checking the accuracy and precision of that algorithm.
  pub struct OrderedDataSet {
    ///The input data
    pub trainning_data: DataSet,
    ///The label data
    pub verification_data: DataSet,
  }
  #[derive(Debug, Serialize, Deserialize, Clone)]
  /// Contains labels and data for machine learning regression.
  pub struct DataSet {
    /// Vec<MouseActionLocations> as a sequence of inputs to a
    /// machine learning model.  MouseActionLocation is the label of
    /// where the next click occured.
    dataset: Vec<(Vec<MouseActionLocation>, MouseActionLocation)>,
  }
  //TODO: verify this
  impl FromIterator<(Vec<MouseActionLocation>, MouseActionLocation)> for DataSet {
    fn from_iter<T>(iter: T) -> Self
    where
      T: IntoIterator<Item = (Vec<MouseActionLocation>, MouseActionLocation)>,
    {
      let mut dataset = Vec::new();
      for (input, output) in iter {
        dataset.push((input, output));
      }
      DataSet { dataset }
    }
  }
  impl IntoIterator for DataSet {
    type Item = (Vec<MouseActionLocation>, MouseActionLocation);
    type IntoIter = std::vec::IntoIter<(Vec<MouseActionLocation>, MouseActionLocation)>;
    fn into_iter(self) -> Self::IntoIter {
      self.dataset.into_iter()
    }
  }

  impl DataSet {
    /// randomize a verification and trainning dataset from a DataSet and return.
    ///
    /// call k times for true k-fold cross validation.
    pub fn k_fold_cross_validation(self, k: usize) -> OrderedDataSet {
      let mut rng = rand::thread_rng();
      //TODO: check replacement in sampling
      //shuffle the data
      let inputs = self
        .dataset
        .iter()
        .choose_multiple(&mut rng, self.dataset.len());
      //return a k sample
      let validation = inputs
        .iter()
        .take(k)
        .cloned()
        .cloned()
        .choose_multiple(&mut rng, k);
      //remove the validation data from the training data
      let inputs = inputs.into_iter().skip(k).cloned().collect();
      let validation = DataSet {
        dataset: validation,
      };

      OrderedDataSet {
        trainning_data: inputs,
        verification_data: validation,
      }
    }
  }

  ///data wrangling method to get clicks as labels and inputs.
  ///
  ///we dont return a bool for label since this should go directly into the network
  ///but we normalize all inputs as a best practice for any ANN.
  ///
  ///tensor types arent used to make this more modular for future frameworks.
  ///
  ///PARAMETERS:
  /// - screen size: length and width in pixels.
  ///
  ///RETURNS:
  /// - a vector of mouse actions ending in a click to be used as a sequence prediction.
  pub fn get_data(
    screen_size: (f32, f32),
  ) -> Result<Vec<(Vec<MouseActionLocation>, MouseActionLocation)>, Error> {
    //assert screen size is positive
    assert!(screen_size.0 > 0.0, "screen width must be positive");
    assert!(screen_size.1 > 0.0, "screen height must be positive");

    //read in the data file rat_nest
    let mut file = File::open("rat_nest").unwrap();
    //cast each line to a MouseAction
    let mut data = Vec::new();
    let mut res = Vec::new();
    let lines = io::BufReader::new(file).lines();
    for line in lines {
      if line.is_err() {
        break;
      }
      let line = line.unwrap();

      let mut line_split = line.split(",");
      //TODO: dont panic
      let x = line_split.next().unwrap().parse::<f32>().unwrap();
      let y = line_split.next().unwrap().parse::<f32>().unwrap();
      let is_clicked = line_split.next().unwrap().parse::<f32>().unwrap();
      let is_fast = line_split.next().unwrap().parse::<f32>().unwrap();
      let is_slow = line_split.next().unwrap().parse::<f32>().unwrap();
      let is_rat_on = line_split.next().unwrap().parse::<f32>().unwrap();
      //normalize x and y
      let x = x / screen_size.0;
      let y = y / screen_size.1;

      let location = MouseActionLocation {
        location: (x, y),
        is_fast: is_fast,
        is_slow: is_slow,
        is_rat_on: is_rat_on,
      };

      let data_entry = (location);
      if is_clicked == 1.0 {
        // an input and label ready to be cast to a tensor
        res.push((data.clone(), location));
        data.clear();
      } else {
        data.push(data_entry);
      }
    }
    //TODO: remove panic, need to resolve recoverable error in at least one calling instance
    assert!(data.len() != 0);

    Ok(res)
  }
}

//write tests
#[cfg(test)]
mod tests {
  use crate::data_logger::get_data;
  #[test]
  fn test_data_wrangling() {
    //call get_data
    let data = get_data((1280.0, 720.0)).unwrap();
    for (input, label) in data {
      //print the input and label
      println!("got data: {:?}\n", input);
      println!("got label: {:?}\n\n", label);
    }
  }
}

//MACHINE LEARNING

//define trainning and inference routines for the rat brain
//the model gets passed in from main where it is checkpointed with
//serialization but the methods to do so are defined here

//start out simple with trainning on the history buffer every
//time a click occurs. episodes are simply the history buffer size
//as a fifo queue.

//training routine for the rat brain
//takes the model and the history buffer
//and trains the model on the history buffer
// pub fn train_model(nodel: &mut Tensor, history: &mut Vec<MouseAction>) {}
