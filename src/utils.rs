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

pub mod io {
    use std::io::{Read, Result, Write};

    pub struct ReadWithProgress<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> {
        read: &'a mut T,
        size: usize,
        progress: Option<F>
    }

    impl<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> ReadWithProgress<'a, T, F> {
        pub fn new(read: &'a mut T, size: usize, progress: Option<F>) -> Self {
            ReadWithProgress { read: read, size: size, progress: progress }
        }
    }

    impl<'a, T: Read + 'a, F: FnMut(usize, usize) -> () + 'a> Read for ReadWithProgress<'a, T, F> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let result = self.read.read(buf);
            if self.progress.is_some() && result.is_ok() {
                let delta = *result.as_ref().unwrap();
                let mut progress = self.progress.as_mut().unwrap();
                progress(self.size, delta);
            }
            result
        }
    }

    pub struct WriteWithProgress<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> {
        write: &'a mut T,
        size: usize,
        progress: Option<F>
    }

    impl<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> WriteWithProgress<'a, T, F> {
        pub fn new(write: &'a mut T, size: usize, progress: Option<F>) -> Self {
            WriteWithProgress { write: write, size: size, progress: progress }
        }
    }

    impl<'a, T: Write + 'a, F: FnMut(usize, usize) -> () + 'a> Write for WriteWithProgress<'a, T, F> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let result = self.write.write(buf);
            if self.progress.is_some() && result.is_ok() {
                let delta = *result.as_ref().unwrap();
                let mut progress = self.progress.as_mut().unwrap();
                progress(self.size, delta);
            }
            result
        }

        fn flush(&mut self) -> Result<()> {
            self.write.flush()
        }
    }

 }