use syn::{parse::Parse, Attribute, Lit};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttributeParseError {
    #[error("some syn error")]
    SynError(#[from] syn::Error),
}

type Result<T> = std::result::Result<T, AttributeParseError>;

pub(crate) enum FieldAttribute {
    Ref(Option<String>),
    Config(String),
    Default,
}

impl FieldAttribute {
    pub fn from_attributes(attrs: &[Attribute]) -> Result<Self> {
        for attr in attrs {
            if attr.path().is_ident("ref") {
                let attr = attr.parse_args::<RefAttr>()?;
                return Ok(FieldAttribute::Ref(attr.0));
            } else if attr.path().is_ident("config") {
                let attr = attr.parse_args::<ConfigAttr>()?;
                return Ok(FieldAttribute::Config(attr.0));
            }
        }
        return Ok(FieldAttribute::Default);
    }
}

struct RefAttr(Option<String>);
struct ConfigAttr(String);

impl Parse for RefAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        if content.is_empty() {
            Ok(RefAttr(None))
        } else {
            content.parse::<Lit>().and_then(|lit| {
                if let Lit::Str(lit_str) = lit {
                    Ok(RefAttr(Some(lit_str.value())))
                } else {
                    Err(syn::Error::new(lit.span(), "expected string literal"))
                }
            })
        }
    }
}

impl Parse for ConfigAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        content.parse::<Lit>().and_then(|lit| {
            if let Lit::Str(lit_str) = lit {
                Ok(ConfigAttr(lit_str.value()))
            } else {
                Err(syn::Error::new(lit.span(), "expected string literal"))
            }
        })
    }
}
