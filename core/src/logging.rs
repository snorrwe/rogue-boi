use std::{
    ops::{Deref, DerefMut},
    sync::Mutex,
};

lazy_static::lazy_static! {
    static ref LOG_BUFFER: Mutex<String> = Default::default();
}

pub(crate) fn get_log_buffer() -> impl Deref<Target = String> + DerefMut<Target = String> {
    LOG_BUFFER.lock().unwrap()
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
