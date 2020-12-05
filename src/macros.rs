#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

#[macro_export]
macro_rules! add_to_activity_list {
    ($cp_state:expr, $($arg:tt)*) => {
        let mut s: String<U60> = String::new();
        uwrite!(s, $($arg)*).ok();
        $cp_state.activity_list.push_back(s);
    };
}
