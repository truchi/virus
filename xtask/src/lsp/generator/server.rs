use super::super::{
    generated::schema::{
        AndType, BaseType, BaseTypes, MessageDirection, Notification, Request, Type,
    },
    utils::{comment_box, docs, ident, pascal, pretty},
    Model, Quote, NOTIFICATIONS, REQUESTS,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

fn usage(type_: &Type, name: &str, model: &Model) -> TokenStream {
    let (_, _, usage, _) = Quote(type_).quote(name, model).unwrap();

    usage
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn quote(model: &Model) -> String {
    let requests = model
        .model
        .requests
        .iter()
        .filter(|request| {
            request.deprecated.is_none()
                && request.proposed.is_none()
                && docs(None, "", request.documentation.as_deref()).is_some()
        })
        .cloned()
        .map(|request| {
            let params = request.params.as_ref().and_then(|params| {
                assert!(params.subtype_1.is_none());
                params.subtype_0.clone()
            });
            let pascal_method = pascal(&request.method.trim_start_matches("$/").replace('/', "_"));

            (request, params, pascal_method)
        })
        .collect::<Vec<_>>();
    let notifications = model
        .model
        .notifications
        .iter()
        .filter(|notification| {
            notification.message_direction != MessageDirection::ClientToServer
                && notification.deprecated.is_none()
                && notification.proposed.is_none()
                && docs(None, "", notification.documentation.as_deref()).is_some()
        })
        .cloned()
        .map(|notification| {
            let params = notification.params.as_ref().and_then(|params| {
                assert!(params.subtype_1.is_none());
                params.subtype_0.clone()
            });
            let pascal_method = pascal(
                &notification
                    .method
                    .trim_start_matches("$/")
                    .replace('/', "_"),
            );

            (notification, params, pascal_method)
        })
        .collect();
    let mod_name = ident(REQUESTS);
    let helpers = quote! {
        use super::#mod_name::*;

        fn missing_parameters() -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing parameters")
        }

        fn missing_result() -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing result")
        }

        fn unknown_method() -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown method")
        }

        fn deserialize_error<T: DeserializeOwned>(error: Error<Value>) -> std::io::Result<Error<T>> {
            Ok(Error {
                code: error.code,
                message: error.message,
                data: error
                    .data
                    .map(|data| serde_json::from_value(data))
                    .transpose()?,
            })
        }

        fn empty_error(error: Error<Value>) -> Error<()> {
            debug_assert!(error.data.is_none());
            Error {
                code: error.code,
                message: error.message,
                data: None,
            }
        }
    };

    pretty(helpers)
        + "\n"
        + &notification(model, notifications)
        + "\n"
        + &request(
            model,
            requests
                .clone()
                .into_iter()
                .filter(|(request, _, _)| {
                    request.message_direction != MessageDirection::ClientToServer
                })
                .collect(),
        )
        + "\n"
        + &response(
            model,
            requests
                .into_iter()
                .filter(|(request, _, _)| {
                    request.message_direction != MessageDirection::ServerToClient
                })
                .collect(),
        )
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn notification(model: &Model, notifications: Vec<(Notification, Option<Type>, Ident)>) -> String {
    let name = pascal("ServerNotification");
    let comment_box = comment_box(&name.to_string());
    let documentation = docs(None, "", "Parameters of a server-to-client notification.").unwrap();
    let variants = notifications.iter().map(|(_, params, pascal_method)| {
        let documentation =
            format!(" @see [`{pascal_method}`](super::{NOTIFICATIONS}::{pascal_method}).");

        if let Some(params) = params {
            assert!(matches!(params, Type::ReferenceType(_)));
            let usage = usage(params, "", model);

            quote! {
                #[doc = #documentation]
                #pascal_method(#usage),
            }
        } else {
            quote! {
                #[doc = #documentation]
                #pascal_method,
            }
        }
    });
    let enumeration = quote! {
        #documentation
        #[derive(Clone, PartialEq, Debug)]
        pub enum #name {
            #(#variants)*
        }
    };
    let deserialize = notifications
        .iter()
        .map(|(notification, params, pascal_method)| {
            let method = &notification.method;
            let deserialize = if params.is_some() {
                quote! {
                    Self::#pascal_method(serde_json::from_value(
                        notification.params.ok_or_else(missing_parameters)?,
                    )?)
                }
            } else {
                quote! {
                    debug_assert!(request.params.is_none());
                    Self::#pascal_method
                }
            };

            quote! {
                #method => { #deserialize },
            }
        });
    let implementation = quote! {
        impl #name {
            pub fn deserialize(notification: Notification<Value>) -> std::io::Result<Self> {
                Ok(match notification.method.as_ref() {
                    #(#deserialize)*
                    _ => return Err(unknown_method()),
                })
            }
        }
    };

    comment_box + "\n" + &pretty(enumeration) + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn request(model: &Model, requests: Vec<(Request, Option<Type>, Ident)>) -> String {
    let name = pascal("ServerRequest");
    let comment_box = comment_box(&name.to_string());
    let documentation = docs(None, "", "Parameters of a server-to-client request.").unwrap();
    let variants = requests.iter().map(|(_, params, pascal_method)| {
        let documentation =
            format!(" @see [`{pascal_method}`](super::{REQUESTS}::{pascal_method}).");

        if let Some(params) = params {
            assert!(matches!(params, Type::ReferenceType(_)));
            let usage = usage(params, "", model);

            quote! {
                #[doc = #documentation]
                #pascal_method(Id, #usage),
            }
        } else {
            quote! {
                #[doc = #documentation]
                #pascal_method(Id),
            }
        }
    });
    let enumeration = quote! {
        #documentation
        #[derive(Clone, PartialEq, Debug)]
        pub enum #name {
            #(#variants)*
        }
    };
    let deserialize = requests.iter().map(|(request, params, pascal_method)| {
        let method = &request.method;
        let deserialize = if params.is_some() {
            quote! {
                Self::#pascal_method(
                    request.id,
                    serde_json::from_value(request.params.ok_or_else(missing_parameters)?)?,
                )
            }
        } else {
            quote! {
                debug_assert!(request.params.is_none());
                Self::#pascal_method(request.id)
            }
        };

        quote! {
            #method => { #deserialize },
        }
    });
    let implementation = quote! {
        impl #name {
            pub fn deserialize(request: Request<Value>) -> std::io::Result<Self> {
                Ok(match request.method.as_ref() {
                    #(#deserialize)*
                    _ => return Err(unknown_method()),
                })
            }
        }
    };

    comment_box + "\n" + &pretty(enumeration) + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn response(model: &Model, requests: Vec<(Request, Option<Type>, Ident)>) -> String {
    let name = pascal("ServerResponse");
    let comment_box = comment_box(&name.to_string());
    let documentation = docs(None, "", "Result or error of a client-to-server request.").unwrap();
    let variants = requests.iter().map(|(request, _, pascal_method)| {
        let documentation =
            format!(" @see [`{pascal_method}`](super::{REQUESTS}::{pascal_method}).");

        let ok = usage(&request.result, &format!("{pascal_method}Result"), model);
        let err = match &request.error_data {
            Some(type_) => usage(type_, &format!("{pascal_method}Error"), model),
            None => quote! { () },
        };

        quote! {
            #[doc = #documentation]
            #pascal_method(Option<Id>, Result<#ok, Error<#err>>),
        }
    });
    let deserialize = requests.iter().map(|(request, _, pascal_method)| {
        let method = &request.method;
        let ok_default = match &request.result {
            Type::AndType(AndType { items, kind })
                if kind == "or"
                    && items
                        .iter()
                        .find(|item| match item {
                            Type::BaseType(BaseType { name, .. }) if *name == BaseTypes::Null => {
                                true
                            }
                            _ => false,
                        })
                        .is_some() =>
            {
                let ok = usage(&request.result, &format!("{pascal_method}Result"), model);
                Some(quote! { #ok::Null(Null) })
            }
            Type::BaseType(BaseType { name, .. }) if *name == BaseTypes::Null => {
                Some(quote! { Null })
            }
            // NOTE we could also check through type aliases...
            _ => None,
        };
        let ok = if let Some(ok_default) = ok_default {
            quote! {
                Ok(response
                    .result
                    .map(|result| serde_json::from_value(result))
                    .transpose()?
                    .unwrap_or(#ok_default))
            }
        } else {
            quote! {
                Ok(serde_json::from_value(
                    response.result.ok_or_else(missing_result)?,
                )?)
            }
        };
        let err = if request.error_data.is_some() {
            quote! { Err(deserialize_error(error)?) }
        } else {
            quote! { Err(empty_error(error)) }
        };

        quote! {
            #method => {
                Self::#pascal_method(
                    response.id,
                    if let Some(error) = response.error {
                        debug_assert!(response.result.is_none());
                        #err
                    } else {
                        #ok
                    },
                )
            },
        }
    });
    let enumeration = quote! {
        #documentation
        #[derive(Clone, PartialEq, Debug)]
        pub enum #name {
            #(#variants)*
        }
    };
    let implementation = quote! {
        impl #name {
            pub fn deserialize(response: Response<Value, Value>, method: &str) -> std::io::Result<Self> {
                Ok(match method {
                    #(#deserialize)*
                    _ => return Err(unknown_method()),
                })
            }
        }
    };

    comment_box + "\n" + &pretty(enumeration) + "\n" + &pretty(implementation)
}
