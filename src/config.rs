pub struct Config {
    pub silent: bool,
    pub verbose: bool,
}

impl Config {
    pub fn new(silent: bool, verbose: bool) -> Self {
        Config { silent, verbose }
    }

    pub fn print_normal(&self, message: &str) {
        if !self.silent {
            println!("{}", message);
        }
    }

    pub fn print_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    pub fn print_warning(&self, message: &str) {
        if !self.silent {
            eprintln!("Warning: {}", message);
        }
    }

    pub fn print_error(&self, message: &str) {
        eprintln!("Error: {}", message); // Errors should always be printed unless explicitly suppressed by a higher-level system
    }
}