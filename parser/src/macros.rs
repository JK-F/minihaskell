#[macro_export]
macro_rules! info_parse {
    ($name:expr, $pair:expr) => {
        info!(
            "Parsing {} {:?}: {:?}",
            $name,
            &$pair.as_rule(),
            &$pair.as_str()
        );
    };
}

