#[macro_export]
macro_rules! write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        $(
            $arg.fmt($dst);
        )+
    }}
}
