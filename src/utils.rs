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

    pub fn verbose<T: Into<String>>(msg: T) {
        if is_relevant(Verbosity::VERBOSE) { print!("{}", Cyan.paint(msg.into())) }
        let _ = std::io::stdout().flush();
    }

    pub fn verboseln<T: Into<String>>(msg: T) {
        if is_relevant(Verbosity::VERBOSE) { println!("{}", Cyan.paint(msg.into())) }
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
        msgln(json);
        Ok(())
    }
}

pub mod time {
    use std::time::{Duration, UNIX_EPOCH, SystemTime};
    use humantime;

    error_chain! {
        errors {
            FailedToParseDuration {
                description("Failed to parse duration")
                display("Failed to parse duration")
            }
        }
    }
    pub fn parse_duration(since: &str) -> Result<Duration> {
        let duration: Duration = humantime::parse_duration(since).chain_err(|| ErrorKind::FailedToParseDuration)?;
        let now = SystemTime::now();
        let history = (now - duration).duration_since(UNIX_EPOCH).chain_err(|| ErrorKind::FailedToParseDuration)?;

        Ok(history)
    }

    pub fn parse_duration_to_unix_ts(since: &str) -> Result<u64> {
        let unix_ts = parse_duration(since)?.as_secs();

        Ok(unix_ts)
    }
}
