use syn::{Attribute, LitStr, Meta, parse::Parse, Type};

pub(crate) enum FieldAttribute {
    Ref(Option<Type>),
    Config(String),
    Default,
}

impl FieldAttribute {
    pub fn from_attributes(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        for attr in attrs {
            if attr.path().is_ident("inject") {
                let maybe_type = if let Meta::Path(_) = attr.meta {
                    None
                } else {
                    let attr = attr.parse_args::<BeanRef>()?;
                    attr.0
                };
                return Ok(FieldAttribute::Ref(maybe_type));
            } else if attr.path().is_ident("value") {
                let attr = attr.parse_args::<ConfigAttr>()?;
                return Ok(FieldAttribute::Config(attr.0));
            }
        }
        return Ok(FieldAttribute::Default);
    }
}

struct BeanRef(Option<Type>);

impl Parse for BeanRef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(BeanRef(None))
        } else {
            let name = input.parse::<Type>()?;
            Ok(BeanRef(Some(name)))
        }
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
        let path = input.parse::<LitStr>()?.value();
        Ok(ConfigAttr(path))
    }
}

pub(crate) struct TypeAttribute {
    pub name: Option<String>,
}

impl TypeAttribute {
    pub fn from_attributes(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        for attr in attrs {
            if attr.path().is_ident("name") {
                let attr = attr.parse_args::<MaybeString>()?;
                return Ok(TypeAttribute { name: attr.0 });
            }
        }
        return Ok(TypeAttribute { name: None });
    }
}