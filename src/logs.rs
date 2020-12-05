
#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {{
        stdout().write_all(b"[INFO] ").unwrap();
        stdout().flush().unwrap();
        println!($($args)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => {unsafe {
        let h_console = GetStdHandle(STD_OUTPUT_HANDLE);
        SetConsoleTextAttribute(h_console, FOREGROUND_RED | FOREGROUND_GREEN);
        stdout().write_all(b"[WARN] ").unwrap();
        stdout().flush().unwrap();
        SetConsoleTextAttribute(h_console, FOREGROUND_RED| FOREGROUND_GREEN | FOREGROUND_BLUE);
        println!($($args)*);
    }};
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {unsafe {
        let h_console = GetStdHandle(STD_OUTPUT_HANDLE);
        SetConsoleTextAttribute(h_console, FOREGROUND_RED | FOREGROUND_INTENSITY);
        stdout().write_all(b"[ERROR] ").unwrap();
        stdout().flush().unwrap();
        SetConsoleTextAttribute(h_console, FOREGROUND_RED| FOREGROUND_GREEN | FOREGROUND_BLUE);
        println!($($args)*);
        crate::utils::pause();
        exit(1);
    }};
}