use log::info;
use typechecker::typecheck;

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

macro_rules! test_typecheck_negatives {
    ($($name:ident: $file:expr,)*) => {
    $(
        #[test]
        fn $name(){
            let _ = env_logger::try_init();
            let src = include_str!($file);
            info!("Reading file: {:?}", $file);
            let program = parser::parse(src).unwrap();
            let typechecked = typecheck(&program);
            assert!(typechecked.is_err(), "Allows for incorrect typechecking!");
        }
    )*
    }
}

test_typecheck! {
    type_alias: "files/type_alias.hs",
    basic_types: "files/basic_types.hs",
    list_types: "files/list_types.hs",
    higher_order_types: "files/higher_order_types.hs",
    id: "files/id.hs",
    letin: "files/let.hs",
    lambda: "files/lambda.hs",
    polymorphic_decl: "files/polymorphic_decl.hs",
    polymorphic_tuple: "files/polymorphic_tuple.hs",
    polymorphic_let: "files/polymorphic_let.hs",
}

test_typecheck_negatives! {
    negative_arg_types_direct: "files/negative_arg_types_direct.hs",
    negative_arg_types_indirect: "files/negative_arg_types_indirect.hs",
}
