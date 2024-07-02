#![allow(unused)]

use super::generated::schema::*;

fn reference(name: &str) -> ReferenceType {
    ReferenceType {
        kind: "reference".into(),
        name: name.into(),
    }
}

fn base(name: BaseTypes) -> BaseType {
    BaseType {
        kind: "base".into(),
        name,
    }
}

fn array(type_: impl Into<Type>) -> Box<ArrayType> {
    Box::new(ArrayType {
        kind: "array".into(),
        element: type_.into(),
    })
}

fn map(type_: impl Into<Type>) -> MapType {
    MapType {
        kind: "map".into(),
        key: MapKeyType::Variant0 {
            kind: "base".into(),
            name: MapKeyTypeVariant0Name::String,
        },
        value: Box::new(type_.into()),
    }
}

fn and(types: impl IntoIterator<Item = Type>) -> AndType {
    AndType {
        kind: "and".into(),
        items: types.into_iter().collect(),
    }
}

fn or(types: impl IntoIterator<Item = Type>) -> OrType {
    OrType {
        kind: "or".into(),
        items: types.into_iter().collect(),
    }
}

fn tuple(types: impl IntoIterator<Item = Type>) -> TupleType {
    TupleType {
        kind: "tuple".into(),
        items: types.into_iter().collect(),
    }
}

fn literal(
    properties: impl IntoIterator<Item = (&'static str, Type, bool)>,
) -> StructureLiteralType {
    StructureLiteralType {
        kind: "literal".into(),
        value: StructureLiteral {
            deprecated: None,
            documentation: None,
            properties: properties
                .into_iter()
                .map(|(name, type_, optional)| Property {
                    deprecated: None,
                    documentation: None,
                    name: name.into(),
                    optional: Some(optional),
                    proposed: None,
                    since: None,
                    type_: type_.into(),
                })
                .collect(),
            proposed: None,
            since: None,
        },
    }
}

fn alias(name: &str, type_: impl Into<Type>) -> TypeAlias {
    TypeAlias {
        deprecated: None,
        documentation: None,
        name: name.into(),
        proposed: None,
        since: None,
        type_: type_.into(),
    }
}

fn structure(
    name: &str,
    extends: impl IntoIterator<Item = &'static str>,
    properties: impl IntoIterator<Item = (&'static str, Type, bool)>,
) -> Structure {
    Structure {
        deprecated: None,
        documentation: None,
        extends: extends
            .into_iter()
            .map(|extend| reference(extend).into())
            .collect(),
        mixins: vec![],
        name: name.into(),
        properties: properties
            .into_iter()
            .map(|(name, type_, optional)| Property {
                deprecated: None,
                documentation: None,
                name: name.into(),
                optional: Some(optional),
                proposed: None,
                since: None,
                type_: type_.into(),
            })
            .collect(),
        proposed: None,
        since: None,
    }
}

pub fn fake() -> MetaModel {
    MetaModel {
        meta_data: MetaData {
            version: "0".into(),
        },
        notifications: vec![],
        requests: vec![],
        type_aliases: vec![],
        enumerations: vec![],
        structures: vec![],
    }
}
