macro_rules! info {
    ($($arg:tt)*) => {
        {
            use nekos_arch::print;
            use colorz::Colorize;
            print!("{}{}{} {}\n",
                "[".green(),
                "info".cyan().bold(),
                "]:".green(),
                format_args!($($arg)*).green()
            );
        }
    };
}

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use nekos_arch::print;
            use colorz::Colorize;
            print!("{}{}{} {}\n",
                "[".bright_black(),
                "debug".bright_black().bold(),
                "]:".bright_black(),
                format_args!($($arg)*).bright_black()
            );
        }

        #[cfg(not(debug_assertions))]
        {
            _ = format_args!($($arg)*);
        }
    };
}

pub(crate) use debug;
pub(crate) use info;
