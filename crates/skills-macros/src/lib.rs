use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct};

/// Marks a function as a skill entry point
///
/// # Example
/// ```ignore
/// #[skill_main]
/// fn run(ctx: &SkillContext) -> Result<(), SkillError> {
///     ctx.log_info("Hello from skill!");
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn skill_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn _skill_entry() -> i32 {
            match #fn_name() {
                Ok(_) => 0,
                Err(_) => 1,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derives the Skill trait for a struct
#[proc_macro_derive(Skill)]
pub fn derive_skill(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        impl Skill for #name {
            fn name(&self) -> &str {
                stringify!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
