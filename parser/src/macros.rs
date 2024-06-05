#[macro_export]
macro_rules! info_parse {
    ($name:expr, $pair:expr) => {
        info!(
            "Parsing {} {:?}: {:?}",
            $name,
            $pair.as_rule(),
            $pair.as_str()
        );
    };
}

#[macro_export]
macro_rules! test_parse {
    ($($name:ident: $file:expr,)*) => {
    $(
        #[test]
        fn $name(){
            let _ = env_logger::try_init();
            let src = include_str!($file);
            info!("Reading file: {:?}", $file);
            let ast = parse(src);
            assert!(ast.is_ok());
            let ast = ast.unwrap();
            for decl in ast {
                info!("{:?}", decl);
            }
        }
    )*
    }
}
