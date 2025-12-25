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
    };
}
