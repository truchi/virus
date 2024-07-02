use super::super::{
    generated::schema::{Enumeration, EnumerationEntryValue, EnumerationTypeName},
    utils::{comment_box, docs, ident, pascal, pretty},
    Model, Quote,
};
use quote::quote;
use syn::Ident;

impl<'a> Quote<&'a Enumeration> {
    pub fn quote(self, _model: &Model) -> Option<String> {
        if self.0.deprecated.is_some() || self.0.proposed.is_some() {
            return None;
        }

        let type_ = self.type_();
        let enum_name = pascal(&self.0.name);
        let comment_box = comment_box(&enum_name.to_string());

        let enumeration = pretty({
            let documentation = docs(self.0.name.as_ref(), "", self.0.documentation.as_deref())?;
            let variants = self
                .0
                .values
                .iter()
                .flat_map(|variant| {
                    if variant.deprecated.is_some() || variant.proposed.is_some() {
                        return None;
                    }

                    let documentation = docs(None, "", variant.documentation.as_deref())?;
                    let variant_name = pascal(&variant.name);
                    let rename = match variant.value.clone() {
                        EnumerationEntryValue::Number(_) => None,
                        EnumerationEntryValue::String(value) => {
                            Some(quote! { #[serde(rename = #value)] })
                        }
                    };

                    Some(quote! {
                        #documentation
                        #rename
                        #variant_name,
                    })
                })
                .chain(self.is_custom().then(|| {
                    let untagged = (!self.is_integer()).then(|| quote! { #[serde(untagged)] });

                    quote! {
                        /// Custom value.
                        #untagged
                        Custom(#type_)
                    }
                }));
            let type_string = type_.to_string();
            let from = self.is_integer().then(|| {
                if self.is_custom() {
                    quote! { #[serde(from = #type_string)] }
                } else {
                    quote! { #[serde(try_from = #type_string)] }
                }
            });
            let into = self
                .is_integer()
                .then(|| quote! { #[serde(into = #type_string)] });

            quote! {
                #documentation
                #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
                #from
                #into
                pub enum #enum_name {
                    #(#variants)*
                }
            }
        });

        if !self.is_integer() {
            return Some(format!("{comment_box}\n{enumeration}\n"));
        }

        let from = {
            let arms = self.0.values.iter().map(|variant| {
                let variant_name = pascal(&variant.name);
                let value = match (self.0.type_.name, variant.value.clone()) {
                    (EnumerationTypeName::Integer, EnumerationEntryValue::Number(value)) => {
                        let value = value as i32;
                        quote! { #value }
                    }
                    (EnumerationTypeName::Uinteger, EnumerationEntryValue::Number(value)) => {
                        let value = value as u32;
                        quote! { #value }
                    }
                    _ => unreachable!(),
                };

                quote! { #value => #enum_name::#variant_name, }
            });

            if self.is_custom() {
                pretty(quote! {
                    impl From<#type_> for #enum_name {
                        fn from(value: #type_) -> Self {
                            match value {
                                #(#arms)*
                                _ => #enum_name::Custom(value),
                            }
                        }
                    }
                })
            } else {
                let enum_name_string = enum_name.to_string();

                pretty(quote! {
                    impl TryFrom<#type_> for #enum_name {
                        type Error = String;

                        fn try_from(value: #type_) -> Result<Self, String> {
                            Ok(match value {
                                #(#arms)*
                                _ => return Err(format!("Invalid `{}` value: {value}", #enum_name_string)),
                            })
                        }
                    }
                })
            }
        };
        let into = {
            let arms = self.0.values.iter().map(|variant| {
                let variant_name = pascal(&variant.name);
                let value = match (self.0.type_.name, variant.value.clone()) {
                    (EnumerationTypeName::Integer, EnumerationEntryValue::Number(value)) => {
                        let value = value as i32;
                        quote! { #value }
                    }
                    (EnumerationTypeName::Uinteger, EnumerationEntryValue::Number(value)) => {
                        let value = value as u32;
                        quote! { #value }
                    }
                    _ => unreachable!(),
                };

                quote! { #enum_name::#variant_name => #value, }
            });
            let custom = self.is_custom().then(|| {
                quote! { #enum_name::Custom(value) => value, }
            });

            pretty(quote! {
                impl Into<#type_> for #enum_name {
                    fn into(self) -> #type_ {
                        match self {
                            #(#arms)*
                            #custom
                        }

                    }
                }
            })
        };

        Some(format!("{comment_box}\n{enumeration}\n{from}\n{into}\n"))
    }

    fn type_(&self) -> Ident {
        ident(match self.0.type_.name {
            EnumerationTypeName::String => "String",
            EnumerationTypeName::Integer => "i32",
            EnumerationTypeName::Uinteger => "u32",
        })
    }

    fn is_integer(&self) -> bool {
        matches!(
            self.0.type_.name,
            EnumerationTypeName::Integer | EnumerationTypeName::Uinteger,
        )
    }

    fn is_custom(&self) -> bool {
        self.0.supports_custom_values == Some(true)
    }
}
