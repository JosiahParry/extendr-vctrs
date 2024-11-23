use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_attribute]
pub fn extendr_vctr(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the struct definition
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Get the class name dynamically (use class() method)
    let class_name = get_class_name(struct_name);
    let fmt_method = format!("format.{class_name}");
    let len_method = format!("length.{class_name}");
    let c_method = format!("c.{class_name}");
    let brack_method = format!("`[.{class_name}`");

    // Generate the necessary code for functions and TryFrom
    let expanded = quote! {
        // Ensure struct is in scope
        #input

        // Methods for show, length, etc.
        // ///@export #fmt_method
        #[extendr(r_name = #fmt_method)]
        pub fn show_vctrsrs(x: #struct_name) -> Strings {
            x.show()
        }

        #[extendr(r_name = #len_method)]
        pub fn length_vctrsrs(x: #struct_name) -> Rint {
            x.length()
        }

        #[extendr(r_name = #brack_method)]
        pub fn subset_vctrsrs(x: #struct_name, idx: Either<Doubles, Integers>) -> Robj {
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

        #[extendr(r_name = #c_method)]
        pub fn extend_vctrsrs(x: #struct_name, y: #struct_name) -> Robj {
            Vctr::from(x.extend(y)).as_vctr()
        }

        // Implement TryFrom for the container
        impl TryFrom<Robj> for #struct_name
        where
            #struct_name: Rvctr,
        {
            type Error = extendr_api::Error;

            fn try_from(value: Robj) -> Result<Self> {
                let inner = Integers::try_from(value)?;
                let pntr = match inner.get_attrib("extendr_ptr") {
                    Some(p) => p,
                    None => return Err(Error::ExpectedExternalPtr(().into())),
                };

                // try and get the data from the pointer
                let extptr = ExternalPtr::<#struct_name>::try_from(pntr)?;
                // make it owned
                let dat = extptr.as_ref().clone();
                Ok(dat)
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper function to get the class name from the struct's class() method.
fn get_class_name(struct_name: &Ident) -> proc_macro2::TokenStream {
    // Generate the class name as a string literal
    quote! { stringify!(#struct_name) }
}
