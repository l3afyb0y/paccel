//! Provides functions to allow us to get
//! the last immediate input speed of the user's mouse movement.
//!
//! This is mainly to allow us to visually represent the user's current
//! speed and applied sensitivity.

use std::{
    fs,
    io::Read,
    sync::atomic::{AtomicU64, Ordering},
    thread::{self, JoinHandle},
};

static INPUT_SPEED_BITS: AtomicU64 = AtomicU64::new(0);

use anyhow::Context;

use crate::libpaccel::fixedptc::Fpt;

pub fn read_input_speed() -> f64 {
    f64::from_bits(INPUT_SPEED_BITS.load(Ordering::Relaxed))
}

pub fn setup_input_speed_reader() -> JoinHandle<anyhow::Result<()>> {
    thread::spawn(|| {
        let mut file = fs::File::open("/dev/paccel").context("failed to open /dev/paccel")?;
        let mut buffer = [0u8; 8];

        loop {
            let nread = file
                .read(&mut buffer)
                .expect("failed to read bytes from /dev/paccel");

            let num = match nread {
                4 => {
                    let buffer = buffer
                        .first_chunk::<4>()
                        .expect("failed to grab 4 bytes from the read buffer");
                    i32::from_ne_bytes(*buffer) as i64
                }
                8 => i64::from_ne_bytes(buffer),
                _ => 0,
            };

            let num: f64 = Fpt(num).into();

            INPUT_SPEED_BITS.store(num.to_bits(), Ordering::Relaxed);

            thread::sleep(std::time::Duration::from_millis(8));
        }
    })
}
