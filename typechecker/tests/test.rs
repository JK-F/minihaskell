use typechecker::typecheck;
use log::info;

macro_rules! test_typecheck {
    ($($name:ident: $file:expr,)*) => {
    $(
        #[test]
        fn $name(){
            let _ = env_logger::try_init();
            let src = include_str!($file);
            info!("Reading file: {:?}", $file);
            let program = parser::parse(src).unwrap();
            let typechecked = typecheck(&program);
            assert!(typechecked.is_ok());
        }
    )*
    }
}

test_typecheck! {
    type_alias: "files/type_alias.hs",
    basic_types: "files/basic_types.hs",
    list_types: "files/list_types.hs",
    higher_order_types: "files/higher_order_types.hs",
}
