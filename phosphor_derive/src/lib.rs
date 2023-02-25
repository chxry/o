use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Item, ItemStruct, ItemEnum, AttributeArgs, NestedMeta, Meta, Ident};

fn get_crate() -> Ident {
  match std::env::var("CARGO_PKG_NAME").unwrap().as_str() {
    "phosphor" => format_ident!("crate"),
    _ => format_ident!("phosphor"),
  }
}

fn enum_struct<F: Fn(TokenStream2, Ident) -> TokenStream2>(
  input: TokenStream,
  f: F,
) -> TokenStream {
  let i = input.clone();
  match parse_macro_input!(i) {
    Item::Struct(ItemStruct { ident, .. }) | Item::Enum(ItemEnum { ident, .. }) => {
      f(TokenStream2::from(input), ident).into()
    }
    _ => quote! {compile_error!("#[component] can only be used on structs or enums.");}.into(),
  }
}

#[proc_macro_attribute]
pub fn component(_: TokenStream, input: TokenStream) -> TokenStream {
  enum_struct(input, |input, ident| {
    let phosphor = get_crate();
    let save = format_ident!("{}_SAVE", ident);
    let load = format_ident!("{}_LOAD", ident);
    let var = format_ident!("{}_LOADER", ident);
    quote! {
      #[allow(non_snake_case)]
      fn #save(data: &Box<dyn Any>) -> Vec<u8>{
        #phosphor::bincode::serialize(&data.downcast_ref::<#ident>().unwrap()).unwrap()
      }
      #[allow(non_snake_case)]
      fn #load(data: Vec<u8>, _: &mut #phosphor::assets::Assets) -> Box<dyn std::any::Any> {
        Box::new(#phosphor::bincode::deserialize::<#ident>(&data).unwrap())
      }
      #[allow(non_upper_case_globals)]
      #[#phosphor::linkme::distributed_slice(#phosphor::scene::COMPONENT_LOADERS)]
      static #var: #phosphor::scene::Loader = #phosphor::scene::Loader {
        id: #phosphor::TypeIdNamed::of::<#ident>(),
        save: &#save,
        load: &#load
      };
      #input
    }
  })
}

#[proc_macro_attribute]
pub fn asset(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as AttributeArgs);
  enum_struct(input, |input, ident| match args.first() {
    Some(NestedMeta::Meta(Meta::Path(p))) => {
      let phosphor = get_crate();
      let func = p.get_ident().unwrap();
      let new_func = format_ident!("_{}", func);
      let var = format_ident!("{}_LOADER", ident);
      quote! {
        fn #new_func(path: &str) -> #phosphor::Result<std::rc::Rc<dyn std::any::Any>> {
          Ok(std::rc::Rc::new(#func(path)?))
        }

        #[allow(non_upper_case_globals)]
        #[#phosphor::linkme::distributed_slice(#phosphor::assets::ASSET_LOADERS)]
        static #var: #phosphor::assets::AssetLoader = #phosphor::assets::AssetLoader {
          id: #phosphor::TypeIdNamed::of::<#ident>(),
          loader: &#new_func,
        };
        #input
      }
    }
    _ => quote! {compile_error!("invalid syntax.")},
  })
}
