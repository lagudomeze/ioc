use syn::{parse::Parse, token::Eq, Attribute, LitStr};
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
            if attr.path().is_ident("bean_ref") {
                let attr = attr.parse_args::<MaybeString>()?;
                return Ok(FieldAttribute::Ref(attr.0));
            } else if attr.path().is_ident("config") {
                let attr = attr.parse_args::<ConfigAttr>()?;
                return Ok(FieldAttribute::Config(attr.0));
            }
        }
        return Ok(FieldAttribute::Default);
    }
}

struct MaybeString(Option<String>);
struct ConfigAttr(String);

impl Parse for MaybeString {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(MaybeString(None))
        } else {
            let name = input.parse::<LitStr>()?.value();
            Ok(MaybeString(Some(name)))
        }
    }
}

impl Parse for ConfigAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Eq>()?;
        let path = input.parse::<LitStr>()?.value();
        Ok(ConfigAttr(path))
    }
}

pub(crate) struct TypeAttribute {
    pub name: Option<String>,
}

impl TypeAttribute {
    pub fn from_attributes(attrs: &[Attribute]) -> Result<Self> {
        for attr in attrs {
            if attr.path().is_ident("name") {
                let attr = attr.parse_args::<MaybeString>()?;
                return Ok(TypeAttribute { name: attr.0 });
            }
        }
        return Ok(TypeAttribute { name: None });
    }
}
