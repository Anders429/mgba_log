mod mgba_bindings;

use mgba_logs::{Level, Record};
use std::{
    env,
    ffi::{c_char, c_uchar, c_void, CStr, CString},
    io::stdout,
    path::Path,
};

fn run(rom: &str) -> Vec<Record> {
    // Create new mGBA core for ROM.
    let rom_c_string = CString::new(rom).expect("failed to convert rom name to CString");
    let mgba = unsafe { mgba_bindings::load(rom_c_string.as_ptr() as *mut c_char) };
    if mgba.is_null() {
        panic!("could not initialize mgba core");
    }

    // Execute ROM.
    let mut results = Vec::<Record>::new();
    // Register callback to catch logs.
    unsafe {
        mgba_bindings::set_log_callback(
            mgba,
            generate_c_callback(|message: *mut c_char, level: u8| {
                if let Ok(level) = level.try_into() {
                    results.push(Record {
                        level,
                        message: CStr::from_ptr(message).to_string_lossy().into_owned(),
                    });
                }
            }),
        );
    }
    while !unsafe { mgba_bindings::is_finished(mgba) } {
        unsafe {
            mgba_bindings::step(mgba);
        }
    }

    // Close mGBA core.
    unsafe {
        mgba_bindings::drop(mgba);
    }

    results
}

unsafe fn generate_c_callback<F>(f: F) -> mgba_bindings::callback
where
    F: FnMut(*mut c_char, c_uchar),
{
    let data = Box::into_raw(Box::new(f));

    mgba_bindings::callback {
        callback: Some(call_closure::<F>),
        data: data as *mut _,
        destroy: Some(drop_box::<F>),
    }
}

extern "C" fn call_closure<F>(data: *mut c_void, message: *mut c_char, level: c_uchar)
where
    F: FnMut(*mut c_char, c_uchar),
{
    let callback_ptr = data as *mut F;
    let callback = unsafe { &mut *callback_ptr };
    callback(message, level);
}

extern "C" fn drop_box<T>(data: *mut c_void) {
    unsafe {
        drop(Box::from_raw(data as *mut T));
    }
}

fn main() {
    let rom = env::args().nth(1).expect("no gba rom filename provided");
    if !Path::new(&rom).exists() {
        panic!("{} does not exist", rom);
    }

    let records = run(&rom);

    serde_json::to_writer(stdout(), &records).expect("could not serialize results");
}
