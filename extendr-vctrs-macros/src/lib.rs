use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn extendr_vctr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the class name from the attribute (e.g., "my_class")
    let class_name = attr.to_string().trim_matches('"').to_string();
    let module_name_str = format!("vctr_{}", class_name);
    let module_name = syn::Ident::new(&module_name_str, proc_macro2::Span::call_site());

    // Parse the input as a struct definition
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Generate unique function names based on the `class` name
    let show_fn_name = syn::Ident::new(&format!("show_{}", class_name), struct_name.span());
    let length_fn_name = syn::Ident::new(&format!("length_{}", class_name), struct_name.span());
    let subset_fn_name = syn::Ident::new(&format!("subset_{}", class_name), struct_name.span());
    let extend_fn_name = syn::Ident::new(&format!("extend_{}", class_name), struct_name.span());

    // Generate R method names
    let fmt_method = format!("format.{class_name}");
    let len_method = format!("length.{class_name}");
    let c_method = format!("c.{class_name}");
    let brack_method = format!("`[.{class_name}`");

    // Generate the expanded code
    let expanded = quote! {
        #input // Preserve the original struct

        pub mod #module_name {
            use super::*;
            use extendr_api::prelude::*;

            /// @export
            #[extendr(r_name = #fmt_method)]
            pub fn #show_fn_name(x: #struct_name) -> Strings {
                x.show()
            }

            /// @export
            #[extendr(r_name = #len_method)]
            pub fn #length_fn_name(x: #struct_name) -> Rint {
                x.length()
            }

            /// @export
            #[extendr(r_name = #brack_method)]
            pub fn #subset_fn_name(x: #struct_name, idx: Either<Doubles, Integers>) -> Robj {
                let idx = match idx {
                    Either::Left(dbl) => dbl
                        .into_iter()
                        .map(|di| {
                            if di.is_na() {
                                Rint::na()
                            } else {
                                Rint::from(di.inner() as i32)
                            }
                        })
                        .collect::<Integers>(),
                    Either::Right(ints) => ints,
                };

                Vctr::from(x.subset(idx)).as_vctr()
            }

            /// @export
            #[extendr(r_name = #c_method)]
            pub fn #extend_fn_name(x: #struct_name, y: #struct_name) -> Robj {
                Vctr::from(x.extend(y)).as_vctr()
            }

            // Implement TryFrom for the struct
            impl TryFrom<Robj> for #struct_name
            where
                #struct_name: Rvctr,
            {
                type Error = extendr_api::Error;

                fn try_from(value: Robj) -> Result<Self> {
                    let inner = Integers::try_from(value)?;
                    let pntr = match inner.get_attrib("extendr_ptr") {
                        Some(p) => p,
                        None => return Err(Self::Error::ExpectedExternalPtr(().into())),
                    };

                    let extptr = ExternalPtr::<#struct_name>::try_from(pntr)?;
                    let dat = extptr.as_ref().clone();
                    Ok(dat)
                }
            }

            extendr_module! {
                mod #module_name;
                fn #show_fn_name;
                fn #length_fn_name;
                fn #subset_fn_name;
                fn #extend_fn_name;
            }
        }
    };

    TokenStream::from(expanded)
}
