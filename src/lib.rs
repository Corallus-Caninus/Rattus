// use inputbot::{KeybdKey::*, MouseButton::*};
// use tensorflow::ops::{Assign, Const, MatMul, Placeholder, Variable};
// use tensorflow::{Graph, Session, Tensor};

// //TODO: implement and moddify the example codes first from Rust Tensorflow
// //      and write the model with pythen graph builder and keras.
// struct Rat_Brain {
//     // build tensorflow graph for reading in screen in X11 and mouse data
// // and predicting where the next click will occure.
// }
use serde_derive::{Deserialize, Serialize};

//a single action of the mouse
//TODO serialize and save these to dot file
struct MouseAction {
    //whatever the precision of the monitor is
    location: (i64, i64),
    is_clicked: bool,
}
