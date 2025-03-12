#[macro_export]
macro_rules! write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        $dst.write_fmt($crate::format_args!($($arg),+));
    }}
}

#[macro_export]
macro_rules! format_args {
    ($($value:expr),+ $(,)?) => {
        $crate::arguments::Arguments::new(&[
            $(
                $crate::arguments::Argument::new(&$value)
            ),+
        ])
    }
}

#[macro_export]
macro_rules! best_fitting {
    ($least_expanded:expr, $($tail:expr),+ $(,)?) => {
        $crate::builders::BestFitting::from_arguments_unchecked(
            $crate::format_args!($least_expanded, $($tail),+)
        )
    };
}
