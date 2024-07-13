use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, DeriveInput, LitStr};

fn get_re(attr: &Attribute) -> Result<String, syn::Error> {
    let mut s = String::new();
    attr.parse_nested_meta(|meta| {
        let ident = meta.path.require_ident()?;
        if ident == "re" {
            let value = meta.value()?;
            s = value.parse::<LitStr>().map(|s| s.value())?;
            return Ok(());
        }
        Err(meta.error(format!("unrecognized attribute for regex_with: {}", ident)))
    })?;
    Ok(s)
}

/// Helper macro for deriving the `FromStr` trait for a struct that captures regular expressions.
/// `regex_with` that specifies the regular expression pattern.
///
/// # Attributes
/// * `regex_with`: specifies the regex pattern used for capturing from a string.
///
/// # Example
/// ```rust
/// use regex_with::Capturable;
/// #[derive(Capturable)]
/// #[regex_with(re = "^\\d+")]
/// struct Number;
/// ```
///
/// This would generate a struct capable of capturing numerical strings using the provided regex pattern.
#[proc_macro_derive(Capturable, attributes(regex_with))]
pub fn capturable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let regex_pattern = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("regex_with") {
                Some(get_re(attr).unwrap())
            } else {
                None
            }
        })
        .expect("regex_with attribute missing");

    let name_cap = format_ident!("{}Capturable", name);

    let expanded = quote! {
        mod _regex_with {
            pub(super) mod private {
                use ::regex_with::regex;
                pub struct #name_cap;
                impl ::regex_with::capturable::Capturable for #name_cap {
                    fn captures<'h>(&self, haystack: &'h str) -> Option<(regex::CaptureNames, regex::Captures<'h>)> {
                        static CELL: ::std::sync::OnceLock<regex::Regex> = ::std::sync::OnceLock::new();
                        let re = CELL.get_or_init(|| { regex::Regex::new(#regex_pattern).unwrap() });
                        let names = re.capture_names();
                        re.captures(haystack).map(|captures| (names, captures))
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements the `FromStr` trait for an struct to provide custom string parsing
/// logic that leverages a regex-based serde deserializer. This macro uses `Capturable` and
/// `serde::Deserialize`, which
/// must be derived separately to provide regex capturing functionality.
///
/// # Dependencies
/// This macro depends on the `Capturable` trait being derived with a corresponding `regex_with` attribute
/// to function correctly, as it utilizes the regex pattern specified there.
///
/// # Example
/// ```rust
/// use regex_with::{Capturable, de::FromStr};
/// #[derive(serde::Deserialize, Capturable, FromStr)]
/// #[regex_with(re = "^(?P<id>\\d+)$")]
/// struct Record {
///     id: u32,
/// }
///
/// let record: Record = "123".parse().unwrap();
/// assert_eq!(record.id, 123);
/// ```
///
/// This implementation allows the `Record` struct to be constructed from a string that strictly contains a numerical ID.
#[proc_macro_derive(DeFromStr, attributes(regex_with))]
pub fn de_from_str_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let name_cap = format_ident!("{}Capturable", name);

    let expanded = quote! {
        impl ::std::str::FromStr for #name {
            type Err = ::regex_with::de::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut de = ::regex_with::Deserializer::new(s, _regex_with::private::#name_cap); // You must need derive(Capturable) for this to work
                ::serde::Deserialize::deserialize(&mut de)
            }
        }
    };

    TokenStream::from(expanded)
}
