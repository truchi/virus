use super::{
    generated::schema::{
        AndType, ArrayType, BaseType, BaseTypes, MapKeyType, MapKeyTypeVariant0Name, MapType,
        OrType, ReferenceType, StringLiteralType, StructureLiteralType, TupleType, Type,
    },
    utils::{docs, ident, pascal, snake},
    Model, Quote, SHIT,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a Type> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        match self.0 {
            Type::BaseType(x) => Quote(x).quote(name, model),
            Type::ReferenceType(x) => Quote(x).quote(name, model),
            Type::ArrayType(x) => Quote(x.as_ref()).quote(name, model),
            Type::MapType(x) => Quote(x).quote(name, model),
            Type::AndType(x) => Quote(x).quote(name, model),
            Type::OrType(x) => Quote(x).quote(name, model),
            Type::TupleType(x) => Quote(x).quote(name, model),
            Type::StructureLiteralType(x) => Quote(x).quote(name, model),
            Type::StringLiteralType(x) => Quote(x).quote(name, model),
            Type::IntegerLiteralType(_) => unreachable!(),
            Type::BooleanLiteralType(_) => unreachable!(),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a BaseType> {
    pub fn quote(
        self,
        _name: &str,
        _model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "base");

        let name = match self.0.name {
            BaseTypes::Uri => "Uri",
            BaseTypes::DocumentUri => "DocumentUri",
            BaseTypes::Integer => "Integer",
            BaseTypes::Uinteger => "UInteger",
            BaseTypes::Decimal => "Decimal",
            BaseTypes::RegExp => "RegExp",
            BaseTypes::String => "String",
            BaseTypes::Boolean => "bool",
            BaseTypes::Null => "Null",
        };

        Some((
            None,
            Vec::new(),
            ident(name).to_token_stream(),
            pascal(name),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a ReferenceType> {
    pub fn quote(
        self,
        _name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "reference");

        if model.skip(&self.0.name) {
            return None;
        }

        let name = pascal(&self.0.name);
        let mod_name = model.mod_name(&name);

        Some((None, Vec::new(), quote! { super::#mod_name::#name }, name))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a ArrayType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "array");

        let name = pascal(name);
        let (declaration, dependencies, usage, name) =
            Quote(&self.0.element).quote(&name.to_string(), model)?;

        Some((
            None,
            declaration.into_iter().chain(dependencies).collect(),
            quote! { Vec<#usage> },
            pascal(&format!("{name}List")),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a MapType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "map");

        let name = pascal(name);
        let (key_usage, key_name) = match &self.0.key {
            MapKeyType::Variant0 { kind, name } => {
                assert!(kind == "base");

                let name = match name {
                    MapKeyTypeVariant0Name::Uri => "Uri",
                    MapKeyTypeVariant0Name::DocumentUri => "DocumentUri",
                    MapKeyTypeVariant0Name::String => "String",
                    MapKeyTypeVariant0Name::Integer => "Integer",
                };

                (ident(name).to_token_stream(), pascal(name))
            }
            MapKeyType::Variant1(reference_type) => {
                let (declaration, dependencies, usage, name) =
                    Quote(reference_type).quote("", model)?;
                assert!(declaration.is_none());
                assert!(dependencies.is_empty());

                (usage, name)
            }
        };
        let (value_declaration, value_dependencies, value_usage, value_name) =
            Quote(self.0.value.as_ref()).quote(&name.to_string(), model)?;

        Some((
            None,
            value_declaration
                .into_iter()
                .chain(value_dependencies)
                .collect(),
            quote! { HashMap<#key_usage, #value_usage> },
            pascal(&format!("{key_name}To{value_name}Map")),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a AndType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        match self.0.kind.as_str() {
            "and" => {}
            "or" => {
                return Quote(&OrType {
                    items: self.0.items.clone(),
                    kind: String::from("or"),
                })
                .quote(name, model)
            }
            "tuple" => {
                return Quote(&TupleType {
                    items: self.0.items.clone(),
                    kind: String::from("tuple"),
                })
                .quote(name, model)
            }
            _ => unreachable!(),
        };

        let name = pascal(name);
        let mut names = Vec::new();
        let mut declarations = Vec::new();
        let items = self
            .0
            .items
            .iter()
            .enumerate()
            .flat_map(|(i, item)| {
                let (declaration, dependencies, usage, name) =
                    Quote(item).quote(&format!("{name}{i}{SHIT}"), model)?;
                let snake = snake(&name.to_string());

                names.push(name.to_string());
                declarations.extend(declaration.into_iter().chain(dependencies));

                Some(quote! {
                    #[serde(flatten)]
                    #snake: #usage,
                })
            })
            .collect::<Vec<_>>();
        assert!(!items.is_empty());

        Some((
            Some(quote! {
                #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
                pub struct #name {
                    #(#items)*
                }
            }),
            declarations,
            quote! { #name },
            pascal(&names.join("And")),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

// TODO: we may have to reorder variants for deserialization to give the "best" match
// see TextDocumentContentChangeEvent for an example of how close we are from apocalypse
impl<'a> Quote<&'a OrType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "or");

        let name = pascal(name);
        let mut names = Vec::new();
        let mut declarations = Vec::new();
        let items = self
            .0
            .items
            .iter()
            .enumerate()
            .flat_map(|(i, item)| {
                // HACK
                // We have to do this shit in AndType I guess, but shit don't happen there for now
                let shitty_name = format!("{name}{i}{SHIT}");
                let (declaration, dependencies, usage, name) = match item {
                    Type::ArrayType(_) => {
                        let (_, _, _, good_name) = Quote(item).quote(&shitty_name, model)?;
                        let good_name = good_name.to_string();
                        let good_name = good_name.trim_end_matches("List");
                        Quote(item).quote(&format!("{name}{good_name}"), model)?
                    }
                    Type::MapType(_) => {
                        // Glad this does not happen cause it'll be harder...
                        unreachable!()
                    }
                    Type::StructureLiteralType(_) => {
                        let (_, _, _, good_name) = Quote(item).quote(&shitty_name, model)?;
                        Quote(item).quote(&format!("{name}{good_name}"), model)?
                    }
                    // Do we also have to care about other types? I don't know.
                    _ => Quote(item).quote(&shitty_name, model)?,
                };

                names.push(name.to_string());
                declarations.extend(declaration.into_iter().chain(dependencies));

                Some(quote! { #name(#usage), })
            })
            .collect::<Vec<_>>();
        assert!(!items.is_empty());

        Some((
            Some(quote! {
                #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
                #[serde(untagged)]
                pub enum #name {
                    #(#items)*
                }
            }),
            declarations,
            quote! { #name },
            pascal(&names.join("Or")),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a TupleType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "tuple");

        let name = pascal(name);
        let mut names = Vec::new();
        let items = self
            .0
            .items
            .iter()
            .enumerate()
            .flat_map(|(i, item)| {
                let (declaration, dependencies, usage, name) =
                    Quote(item).quote(&format!("{name}{i}{SHIT}"), model)?;

                // It happens to be like this for now
                // If it's not the case anymore, wwe have to propagate those
                assert!(declaration.is_none());
                assert!(dependencies.is_empty());

                names.push(name.to_string());

                Some(usage)
            })
            .collect::<Vec<_>>();
        assert!(!items.is_empty());

        Some((
            None,
            Vec::new(),
            quote! { (#(#items),*) },
            pascal(&names.join("And")),
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a StructureLiteralType> {
    pub fn quote(
        self,
        name: &str,
        model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "literal");

        if self.0.value.deprecated.is_some() || self.0.value.proposed.is_some() {
            return None;
        }

        let name = pascal(name);
        let documentation = docs(None, "", self.0.value.documentation.as_deref())?;
        let mut names = Vec::new();
        let mut declarations = Vec::new();
        let properties = self
            .0
            .value
            .properties
            .iter()
            .flat_map(|property| {
                if property.deprecated.is_some() || property.proposed.is_some() {
                    return None;
                }

                let property_name = &property.name;
                let snake_name = snake(property_name);
                let pascal_name = pascal(property_name);
                let documentation = docs(None, "", property.documentation.as_deref())?;
                let (declaration, dependencies, type_, _) =
                    Quote(&property.type_).quote(&format!("{name}{pascal_name}"), model)?;
                let type_ = (property.optional == Some(true))
                    .then(|| quote! { Option<#type_> })
                    .unwrap_or(type_);

                names.push(
                    (property.optional == Some(true))
                        .then(|| pascal(&format!("Optional{pascal_name}")))
                        .unwrap_or(pascal_name)
                        .to_string(),
                );
                declarations.extend(declaration.into_iter().chain(dependencies));

                Some(quote! {
                    #documentation
                    #[serde(rename = #property_name)]
                    #snake_name: #type_,
                })
            })
            .collect::<Vec<_>>();

        Some((
            Some(quote! {
                #documentation
                #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
                pub struct #name {
                    #(#properties)*
                }
            }),
            declarations,
            quote! { #name },
            if names.is_empty() {
                // name
                pascal("Empty")
            } else {
                pascal(&names.join("And"))
            },
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl<'a> Quote<&'a StringLiteralType> {
    pub fn quote(
        self,
        name: &str,
        _model: &Model,
    ) -> Option<(Option<TokenStream>, Vec<TokenStream>, TokenStream, Ident)> {
        assert!(self.0.kind == "stringLiteral");

        let name = pascal(name);
        let value = &self.0.value;
        let pascal_value = pascal(value);

        Some((
            Some(quote! {
                #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
                pub enum #name {
                    #[serde(rename = #value)]
                    #pascal_value,
                }
            }),
            Vec::new(),
            quote! { #name },
            name,
        ))
    }
}
