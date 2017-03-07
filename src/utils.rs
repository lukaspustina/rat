pub mod console {
    use config::Verbosity;

    use term_painter::ToStyle;
    use term_painter::Color::*;
    use std;
    use std::io::Write;
    use std::sync::{Once, ONCE_INIT};

    static mut VERBOSITY: Option<Verbosity> = None;
    static INIT: Once = ONCE_INIT;

    pub fn init(verbosity: Verbosity) {
        unsafe {
            INIT.call_once(|| {
                VERBOSITY = Some(verbosity);
            });
        }
    }

    pub fn info<T: Into<String>>(msg: T) {
        if is_relevant(Verbosity::NORMAL) { println!("{}", Blue.paint(msg.into())) }
    }

    pub fn msgln<T: Into<String>>(msg: T) {
        println!("{}", msg.into())
    }

    pub fn msg<T: Into<String>>(msg: T) {
        print!("{}", msg.into());
        let _ = std::io::stdout().flush();
    }

    pub fn error<T: Into<String>>(msg: T) {
        if is_relevant(Verbosity::NORMAL) { println!("{}", Red.paint(msg.into())) }
    }

    pub fn warning<T: Into<String>>(msg: T) {
        if is_relevant(Verbosity::NORMAL) { println!("{}", Yellow.paint(msg.into())) }
    }

    fn is_relevant(my_verbosity: Verbosity) -> bool {
        let verbosity = unsafe { VERBOSITY.unwrap() };
        my_verbosity >= verbosity
    }
}

pub mod output {
    use super::console::msgln;

    error_chain! {
        errors {
           OutputFailed {
                description("Failed to print message")
                display("Failed to print message")
            }
        }
    }

    pub fn as_json(json: &str) -> Result<()> {
        msgln(format!("{}", json));
        Ok(())
    }
}
