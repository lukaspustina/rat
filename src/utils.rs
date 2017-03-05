pub mod console {
    use term_painter::ToStyle;
    use term_painter::Color::*;

    pub fn info<T: Into<String>>(msg: T) {
        println!("{}", Blue.paint(msg.into()))
    }

    pub fn msg<T: Into<String>>(msg: T) {
        println!("{}", msg.into())
    }

    pub fn error<T: Into<String>>(msg: T) {
        println!("{}", Red.paint(msg.into()))
    }

    pub fn warning<T: Into<String>>(msg: T) {
        println!("{}", Yellow.paint(msg.into()))
    }

}
