use std::{
    fmt::Write,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use arrayvec::ArrayVec;

const HISTORY_LEN: usize = 8;

static CURRENT_IND: AtomicUsize = AtomicUsize::new(0);
lazy_static::lazy_static! {
    static ref LOG_BUFFER: [Mutex<String>; HISTORY_LEN] = Default::default();
}

pub(crate) fn rotate_log() {
    let ind = CURRENT_IND.load(Ordering::Relaxed);
    let new_ind = (ind + 1) % HISTORY_LEN;
    LOG_BUFFER[new_ind].lock().unwrap().clear();
    CURRENT_IND
        .compare_exchange(ind, new_ind, Ordering::Relaxed, Ordering::Relaxed)
        .expect("Failed to update index");
}

pub(crate) fn compute_log(current_tick: usize) -> String {
    let mut output = String::with_capacity(1024 * 4);

    let from = current_tick.checked_sub(HISTORY_LEN).unwrap_or(0);

    let mut j = CURRENT_IND.load(Ordering::Relaxed);
    let indices = (from..current_tick)
        .rev()
        .map(|i| {
            let res = (i, j);
            j = j.checked_sub(1).unwrap_or(HISTORY_LEN - 1);
            res
        })
        .collect::<ArrayVec<_, HISTORY_LEN>>();
    for (i, j) in indices.into_iter().rev() {
        writeln!(output, "---------------- tick {} ----------------", i)
            .expect("Failed to write header");
        writeln!(output, "{}", LOG_BUFFER[j].lock().unwrap().as_str())
            .expect("Failed to write log");
    }

    output
}

pub(crate) fn get_log_buffer() -> impl Deref<Target = String> + DerefMut<Target = String> {
    LOG_BUFFER[CURRENT_IND.load(Ordering::Relaxed)]
        .lock()
        .unwrap()
}

#[macro_export]
macro_rules! game_log {
    ($($args: tt),*) => {
        {
            use crate::logging::get_log_buffer;
            use std::fmt::Write;
            writeln!(get_log_buffer(), $($args),*).expect("Failed to append line");
        }
    };
}
