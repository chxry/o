use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Item, ItemStruct, ItemEnum};

#[proc_macro_attribute]
pub fn component(_: TokenStream, input: TokenStream) -> TokenStream {
  let input2 = proc_macro2::TokenStream::from(input.clone());
  match parse_macro_input!(input) {
    Item::Struct(ItemStruct { ident, .. }) | Item::Enum(ItemEnum { ident, .. }) => {
      let phosphor = match std::env::var("CARGO_PKG_NAME").unwrap().as_str() {
        "phosphor" => format_ident!("crate"),
        _ => format_ident!("phosphor"),
      };
      let save = format_ident!("{}_SAVE", ident);
      let load = format_ident!("{}_LOAD", ident);
      let var = format_ident!("{}_LOADER", ident);
      quote! {
        #[allow(non_snake_case)]
        fn #save(data: &Box<dyn Any>) -> Vec<u8>{
          #phosphor::bincode::serialize(&data.downcast_ref::<#ident>().unwrap()).unwrap()
        }
        #[allow(non_snake_case)]
        fn #load(data: Vec<u8>, _: &mut #phosphor::assets::Assets) -> Box<dyn Any> {
          Box::new(#phosphor::bincode::deserialize::<#ident>(&data).unwrap())
        }
        #[allow(non_upper_case_globals)]
        #[#phosphor::linkme::distributed_slice(#phosphor::scene::LOADERS)]
        static #var: #phosphor::scene::Loader = #phosphor::scene::Loader {
          id: #phosphor::TypeIdNamed::of::<#ident>(),
          save: &#save,
          load: &#load
        };
        #input2
      }
    }
    _ => quote! {compile_error!("#[component] can only be used on structs or enums.");},
  }
  .into()
}
