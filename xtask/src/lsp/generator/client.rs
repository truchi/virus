use super::super::{
    generated::schema::{MessageDirection, Notification, Request, Type},
    utils::{comment_box, docs, ident, pascal, pretty, snake},
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
            let method = request.method.trim_start_matches("$/").replace('/', "_");
            let pascal_method = pascal(&method);
            let snake_method = snake(&method);

            (request, params, pascal_method, snake_method)
        })
        .collect::<Vec<_>>();
    let notifications = model
        .model
        .notifications
        .iter()
        .filter(|notification| {
            notification.message_direction != MessageDirection::ServerToClient
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
            let method = notification
                .method
                .trim_start_matches("$/")
                .replace('/', "_");
            let pascal_method = pascal(&method);
            let snake_method = snake(&method);

            (notification, params, pascal_method, snake_method)
        })
        .collect::<Vec<_>>();
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
                .filter(|(request, _, _, _)| {
                    request.message_direction != MessageDirection::ServerToClient
                })
                .collect(),
        )
        + "\n"
        + &response(
            model,
            requests
                .clone()
                .into_iter()
                .filter(|(request, _, _, _)| {
                    request.message_direction != MessageDirection::ClientToServer
                })
                .collect(),
        )
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn notification(
    model: &Model,
    notifications: Vec<(Notification, Option<Type>, Ident, Ident)>,
) -> String {
    let comment_box = comment_box("LspClientNotification");
    let methods = notifications
        .iter()
        .map(|(_, params, pascal_method, snake_method)| {
            let documentation =
                format!(" @see [`{pascal_method}`](super::{NOTIFICATIONS}::{pascal_method}).");

            if let Some(params) = params {
                assert!(matches!(params, Type::ReferenceType(_)));
                let usage = usage(params, "", model);

                quote! {
                    #[doc = #documentation]
                    pub async fn #snake_method(&mut self, params: #usage) -> std::io::Result<()> {
                        self.client.send_notification::<#pascal_method>(params).await
                    }
                }
            } else {
                quote! {
                    #[doc = #documentation]
                    pub async fn #snake_method(&mut self) -> std::io::Result<()> {
                        self.client.send_notification::<#pascal_method>(()).await
                    }
                }
            }
        });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> super::LspClientNotification<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn request(model: &Model, requests: Vec<(Request, Option<Type>, Ident, Ident)>) -> String {
    let comment_box = comment_box("LspClientRequest");
    let methods = requests
        .iter()
        .map(|(request, params, pascal_method, snake_method)| {
            let documentation =
                format!(" @see [`{pascal_method}`](super::{REQUESTS}::{pascal_method}).");
            let ok = usage(&request.result, &format!("{pascal_method}Result"), model);
            let err = match &request.error_data {
                Some(type_) => usage(type_, &format!("{pascal_method}Error"), model),
                None => quote! { () },
            };
            let return_type = quote! {
                std::io::Result<
                    impl '_ + futures::Future<Output = std::io::Result<Result<#ok, Error<#err>>>>
                >
            };

            if let Some(params) = params {
                assert!(matches!(params, Type::ReferenceType(_)));
                let usage = usage(params, "", model);

                quote! {
                    #[doc = #documentation]
                    pub async fn #snake_method(&mut self, params: #usage) -> #return_type {
                        self.client.send_request::<#pascal_method>(params).await
                    }
                }
            } else {
                quote! {
                    #[doc = #documentation]
                    pub async fn #snake_method(&mut self) -> #return_type {
                        self.client.send_request::<#pascal_method>(()).await
                    }
                }
            }
        });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> super::LspClientRequest<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn response(model: &Model, requests: Vec<(Request, Option<Type>, Ident, Ident)>) -> String {
    let comment_box = comment_box("LspClientResponse");
    let methods = requests
        .iter()
        .map(|(request, _, pascal_method, snake_method)| {
            let documentation =
                format!(" @see [`{pascal_method}`](super::{REQUESTS}::{pascal_method}).");
            let ok = usage(&request.result, &format!("{pascal_method}Result"), model);
            let err = match &request.error_data {
                Some(type_) => usage(type_, &format!("{pascal_method}Error"), model),
                None => quote! { () },
            };

            quote! {
                #[doc = #documentation]
                pub async fn #snake_method(
                    &mut self,
                    id: Option<Id>,
                    data: Result<#ok, Error<#err>>,
                ) -> std::io::Result<()> {
                    self.client.send_response::<#pascal_method>(id, data).await
                }
            }
        });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> super::LspClientResponse<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}
