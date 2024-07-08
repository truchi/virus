use super::super::{
    generated::schema::{MessageDirection, Notification, Request, Type},
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
    let uses = {
        let notifications_mod_name = ident(NOTIFICATIONS);
        let requests_mod_name = ident(REQUESTS);

        quote! {
            use super::#requests_mod_name::*;
            use super::#notifications_mod_name::*;
        }
    };

    pretty(uses)
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
                    Self::#pascal_method(
                        <#pascal_method as NotificationTrait>::deserialize(
                            notification,
                        )?,
                    )
                }
            } else {
                quote! {
                    debug_assert!(notification.params.is_none());
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
                    _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown method")),
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
                    request.id.clone(),
                    <#pascal_method as RequestTrait>::deserialize_request(
                        request,
                    )?,
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
                    _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown method")),
                })
            }
        }
    };

    comment_box + "\n" + &pretty(enumeration) + "\n" + &pretty(implementation)
}
