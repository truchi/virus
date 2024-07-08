mod fake;
mod generated {
    pub mod model;
    pub mod schema;
}
mod generator {
    pub mod client;
    pub mod enumerations;
    pub mod notifications;
    pub mod requests;
    pub mod server;
    pub mod structures;
    pub mod type_aliases;
}
mod types;
mod utils;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

use generated::{
    model::{MODEL, VERSION},
    schema::{MetaModel, ReferenceType, Structure, Type},
};
use quote::quote;
use std::{collections::HashMap, process::Command};
use syn::Ident;
use utils::{comment_box, ident, pascal, pretty};

// ────────────────────────────────────────────────────────────────────────────────────────────── //

const HEADER: &str = "// 🚨 This file is generated by `cargo xtask-lsp`\n\n";

const CLIENT: &'static str = "client";
const SERVER: &'static str = "server";
const ENUMERATIONS: &'static str = "enumerations";
const STRUCTURES: &'static str = "structures";
const TYPE_ALIASES: &'static str = "type_aliases";
const NOTIFICATIONS: &'static str = "notifications";
const REQUESTS: &'static str = "requests";

const DEPRECATED: &'static str = "@deprecated";
const PROPOSED: &'static str = "@proposed";

// NOTE
// I have a lot of trouble generating correct names for non top-level types
// I think my hacky code works nice for now, but we are lucky
const SHIT: &'static str = "SHIIIIIIIIIIT";

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let safe_path = |relative: &str| {
        let path = current_dir.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        path
    };
    let model = Model::new(serde_json::from_str::<MetaModel>(MODEL).unwrap());
    let prelude = quote! { use super::*; }.to_string() + "\n\n";
    let bounds = quote! { 'static + Serialize + DeserializeOwned + Send + Sync };
    let generate = |file: &str, content: String| {
        println!("🤖 .......... {file}");
        std::fs::write(
            safe_path(&format!("crates/lsp/src/generated/{file}.rs")),
            format!("{HEADER}{prelude}{content}"),
        )
        .unwrap();
    };

    macro_rules! my_quote {
        ($things:ident) => {
            model
                .model
                .$things
                .iter()
                .flat_map(|thing| Quote(thing).quote(&model))
        };
    }

    //
    // Let's go!
    //

    println!("💡 To update the model, run:");
    println!("💻 ./xtask/src/lsp/generated/generate.sh");
    println!("");
    println!("💘 Generating LSP {VERSION} types");

    generate(SERVER, generator::server::quote(&model));
    generate(CLIENT, generator::client::quote(&model));
    generate(
        ENUMERATIONS,
        my_quote!(enumerations)
            .inspect(|s| assert!(!s.contains(SHIT)))
            .collect::<String>(),
    );
    generate(
        STRUCTURES,
        my_quote!(structures)
            .inspect(|s| assert!(!s.contains(SHIT)))
            .collect::<String>(),
    );
    generate(
        TYPE_ALIASES,
        my_quote!(type_aliases)
            .inspect(|s| assert!(!s.contains(SHIT)))
            .collect::<String>(),
    );
    generate(
        NOTIFICATIONS,
        comment_box("NotificationTrait")
            + "\n"
            + &pretty(quote! {
                fn missing_params() -> std::io::Error {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing params")
                }
            })
            + "\n"
            + &pretty(quote! {
                /// A trait for notifications.
                pub trait NotificationTrait {
                    const REGISTRATION_METHOD: Option<&'static str>;
                    const METHOD: &'static str;
                    type RegistrationOptions: #bounds;
                    type Params: #bounds;

                    fn params(params: Self::Params) -> Option<Self::Params>;
                    fn deserialize(notification: Notification<Value>) -> std::io::Result<Self::Params>;
                }
            })
            + "\n"
            + &my_quote!(notifications)
                .map(|(name, types)| {
                    comment_box(&name.to_string())
                        + "\n"
                        + &types
                            .into_iter()
                            .map(|type_| pretty(type_) + "\n")
                            .collect::<String>()
                })
                .inspect(|s| assert!(!s.contains(SHIT)))
                .collect::<String>(),
    );
    generate(
        REQUESTS,
        comment_box("RequestTrait")
            + "\n"
            + &pretty(quote! {
                fn missing_params() -> std::io::Error {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing params")
                }

                fn missing_result() -> std::io::Error {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing result")
                }
            })
            + "\n"
            + &pretty(quote! {
                /// A trait for requests.
                pub trait RequestTrait {
                    const REGISTRATION_METHOD: Option<&'static str>;
                    const METHOD: &'static str;
                    type RegistrationOptions: #bounds;
                    type Params: #bounds;
                    type PartialResult: #bounds;
                    type Result: #bounds;
                    type Error: #bounds;

                    fn params(params: Self::Params) -> Option<Self::Params>;
                    fn error(error: Error<Self::Error>) -> Error<Self::Error>;
                    fn deserialize_request(request: Request<Value>) -> std::io::Result<Self::Params>;
                    fn deserialize_response(response: Response<Value, Value>) -> std::io::Result<Result<Self::Result, Error<Self::Error>>>;
                }
            })
            + "\n"
            + &my_quote!(requests)
                .map(|(name, types)| {
                    comment_box(&name.to_string())
                        + "\n"
                        + &types
                            .into_iter()
                            .map(|type_| pretty(type_) + "\n")
                            .collect::<String>()
                })
                .inspect(|s| assert!(!s.contains(SHIT)))
                .collect::<String>(),
    );

    println!("");
    println!("🧽 Formatting");

    Command::new("cargo")
        .arg("fmt")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    println!("");
    println!("🎉 Checking lsp crate");

    Command::new("cargo")
        .args(["check", "--package", "lsp"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

struct Quote<T>(pub T);

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct Model {
    model: MetaModel,
    map: HashMap<Ident, (/* mod name */ Ident, /* skip */ bool)>,
}

impl Model {
    fn new(model: MetaModel) -> Self {
        fn skip(documentation: &Option<String>) -> bool {
            documentation
                .as_ref()
                .filter(|documentation| documentation.contains(DEPRECATED))
                .filter(|documentation| documentation.contains(PROPOSED))
                .is_some()
        }

        let structures = model.structures.iter().map(|structure| {
            (
                &structure.name,
                STRUCTURES,
                structure.deprecated.is_some()
                    || structure.proposed.is_some()
                    || skip(&structure.documentation),
            )
        });
        let enumerations = model.enumerations.iter().map(|enumeration| {
            (
                &enumeration.name,
                ENUMERATIONS,
                enumeration.deprecated.is_some()
                    || enumeration.proposed.is_some()
                    || skip(&enumeration.documentation),
            )
        });
        let type_aliases = model.type_aliases.iter().map(|type_alias| {
            (
                &type_alias.name,
                TYPE_ALIASES,
                type_alias.deprecated.is_some()
                    || type_alias.proposed.is_some()
                    || skip(&type_alias.documentation),
            )
        });
        let map = structures
            .chain(enumerations)
            .chain(type_aliases)
            .map(|(name, mod_name, skip)| (pascal(name), (ident(mod_name), skip)))
            .collect();

        Self { model, map }.sanitize()
    }

    fn sanitize(mut self) -> Self {
        // Sort
        self.model
            .enumerations
            .sort_by_key(|enumeration| enumeration.name.clone());
        self.model
            .notifications
            .sort_by_key(|notification| notification.method.clone());
        self.model
            .requests
            .sort_by_key(|request| request.method.clone());
        self.model
            .structures
            .sort_by_key(|structure| structure.name.clone());
        self.model
            .type_aliases
            .sort_by_key(|type_alias| type_alias.name.clone());

        // Flatten `_InitializeParams` and `WorkspaceFoldersInitializeParams`
        // into `InitializeParams`
        let initialize_params = {
            let index = self
                .model
                .structures
                .iter()
                .position(|structure| structure.name == "_InitializeParams")
                .unwrap();
            self.model.structures.remove(index)
        };
        let workspace_folders_initialize_params = {
            let index = self
                .model
                .structures
                .iter()
                .position(|structure| structure.name == "WorkspaceFoldersInitializeParams")
                .unwrap();
            self.model.structures.remove(index)
        };
        assert!(!self.skip("_InitializeParams") && !self.skip("WorkspaceFoldersInitializeParams"));

        let structure = self
            .model
            .structures
            .iter_mut()
            .find(|structure| structure.name == "InitializeParams")
            .unwrap();
        assert!(
            structure.extends.len() == 2
                && matches!(
                    &structure.extends[0],
                    Type::ReferenceType(ReferenceType { name, .. })
                    if name == "_InitializeParams",
                )
                && matches!(
                    &structure.extends[1],
                    Type::ReferenceType(ReferenceType { name, .. })
                    if name == "WorkspaceFoldersInitializeParams",
                )
        );

        structure.extends = initialize_params
            .extends
            .into_iter()
            .chain(initialize_params.mixins)
            .chain(workspace_folders_initialize_params.extends)
            .chain(workspace_folders_initialize_params.mixins)
            .collect();
        structure.properties = initialize_params
            .properties
            .into_iter()
            .chain(workspace_folders_initialize_params.properties)
            .collect();

        self
    }

    fn mod_name(&self, pascal: &Ident) -> &Ident {
        &self.map.get(pascal).unwrap().0
    }

    fn skip(&self, name: &str) -> bool {
        self.map.get(&pascal(name)).unwrap().1
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

impl Structure {
    fn extends<'a>(
        &'a self,
        structures: &'a Vec<Structure>,
    ) -> impl 'a + Iterator<Item = &'a Structure> {
        self.extends
            .iter()
            .chain(self.mixins.iter())
            .map(|extend| match extend {
                Type::ReferenceType(ReferenceType { kind, name }) => {
                    assert!(kind == "reference");
                    structures
                        .iter()
                        .find(|structure| structure.name == *name)
                        .unwrap()
                }
                _ => unreachable!(),
            })
    }
}
