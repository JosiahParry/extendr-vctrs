use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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

        pub(crate) mod #module_name {
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

            impl From<#struct_name> for Robj {
                fn from(value: #struct_name) -> Self {
                    Vctr::from(value).as_vctr()
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

#[proc_macro_derive(Rvctr)]
pub fn derive_rvctr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    // Ensure it's a tuple struct with a single unnamed field
    match &input.data {
        Data::Struct(data) => {
            if let Fields::Unnamed(fields) = &data.fields {
                if fields.unnamed.len() == 1 {
                    &fields.unnamed[0].ty
                } else {
                    return syn::Error::new_spanned(
                        &data.fields,
                        "Rvctr can only be derived for tuple structs with one unnamed field.",
                    )
                    .to_compile_error()
                    .into();
                }
            } else {
                return syn::Error::new_spanned(
                    &data.fields,
                    "Rvctr can only be derived for tuple structs.",
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(&input, "Rvctr can only be derived for structs.")
                .to_compile_error()
                .into();
        }
    };

    // Generate implementation
    let expanded = quote! {
        impl Rvctr for #struct_name {
            fn length(&self) -> Rint {
                extendr_vctrs::helpers::vctr_len(&self.0)
            }

            fn show(&self) -> Strings {
                extendr_vctrs::helpers::vctr_show(&self.0)
            }

            fn subset(&self, idx: Integers) -> Self {
                let new_inner = extendr_vctrs::helpers::vctr_subset(&self.0, idx);
                Self(new_inner)
            }

            fn extend(self, y: Self) -> Self {
                let inner = extendr_vctrs::helpers::vctr_extend(self.0, y.0);
                Self(inner)
            }

            fn class() -> &'static str {
                stringify!(#struct_name)
            }
        }

        impl From<#struct_name> for Robj {
            fn from(value: #struct_name) -> Self {
                Vctr::from(value).as_vctr()
            }
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
    };

    TokenStream::from(expanded)
}
