use super::super::{
    generated::schema::{MessageDirection, Notification, Request, Type},
    utils::{comment_box, docs, ident, pascal, pretty, snake},
    Model, Quote, REQUESTS,
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
            let snake_method = snake(&request.method.trim_start_matches("$/").replace('/', "_"));

            (request, params, snake_method)
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
            let snake_method = snake(
                &notification
                    .method
                    .trim_start_matches("$/")
                    .replace('/', "_"),
            );

            (notification, params, snake_method)
        })
        .collect::<Vec<_>>();
    let mod_name = ident(REQUESTS);
    let helpers = quote! {
        use super::#mod_name::*;
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
                .filter(|(request, _, _)| {
                    request.message_direction != MessageDirection::ClientToServer
                })
                .collect(),
        )
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn notification(model: &Model, notifications: Vec<(Notification, Option<Type>, Ident)>) -> String {
    let comment_box = comment_box("LspClientNotify");
    let methods = notifications
        .iter()
        .map(|(notification, params, snake_method)| {
            let method = &notification.method;
            let documentation = docs(
                Some(method.as_str()),
                match notification.message_direction {
                    MessageDirection::ClientToServer => "➡️ ",
                    MessageDirection::ServerToClient => "⬅️ ",
                    MessageDirection::Both => "➡️ ⬅️ ",
                },
                notification.documentation.as_deref(),
            );

            if let Some(params) = params {
                assert!(matches!(params, Type::ReferenceType(_)));
                let usage = usage(params, "", model);

                quote! {
                    #documentation
                    pub async fn #snake_method(&mut self, params: #usage) -> std::io::Result<()> {
                        self.client.send_notification(Cow::Borrowed(#method), Some(params)).await
                    }
                }
            } else {
                quote! {
                    #documentation
                    pub async fn #snake_method(&mut self) -> std::io::Result<()> {
                        self.client.send_notification::<()>(Cow::Borrowed(#method), None).await
                    }
                }
            }
        });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> LspClientNotify<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn request(model: &Model, requests: Vec<(Request, Option<Type>, Ident)>) -> String {
    let comment_box = comment_box("LspClientRequest");
    let methods = requests.iter().map(|(request, params, snake_method)| {
        let method = &request.method;
        let documentation = docs(
            Some(method.as_str()),
            match request.message_direction {
                MessageDirection::ClientToServer => "↩️ ",
                MessageDirection::ServerToClient => "↪️ ",
                MessageDirection::Both => "↩️ ↪️ ",
            },
            request.documentation.as_deref(),
        );

        if let Some(params) = params {
            assert!(matches!(params, Type::ReferenceType(_)));
            let usage = usage(params, "", model);

            quote! {
                #documentation
                pub async fn #snake_method(&mut self, params: #usage) -> std::io::Result<Id> {
                    self.client.send_request(Cow::Borrowed(#method), Some(params)).await
                }
            }
        } else {
            quote! {
                #documentation
                pub async fn #snake_method(&mut self) -> std::io::Result<Id> {
                    self.client.send_request::<()>(Cow::Borrowed(#method), None).await
                }
            }
        }
    });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> LspClientRequest<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn response(model: &Model, requests: Vec<(Request, Option<Type>, Ident)>) -> String {
    let comment_box = comment_box("LspClientRespond");
    let methods = requests.iter().map(|(request, _, snake_method)| {
        let method = &request.method;
        let documentation = docs(
            Some(method.as_str()),
            match request.message_direction {
                MessageDirection::ClientToServer => "↩️ ",
                MessageDirection::ServerToClient => "↪️ ",
                MessageDirection::Both => "↩️ ↪️ ",
            },
            request.documentation.as_deref(),
        );
        let pascal_method = pascal(&snake_method.to_string());
        let ok = usage(&request.result, &format!("{pascal_method}Result"), model);
        let err = match &request.error_data {
            Some(type_) => usage(type_, &format!("{pascal_method}Error"), model),
            None => quote! { () },
        };

        quote! {
            #documentation
            pub async fn #snake_method(&mut self, id: Option<Id>, data: Result<#ok, Error<#err>>) -> std::io::Result<()> {
                self.client.send_response(id, data).await
            }
        }
    });
    let implementation = quote! {
        impl<'client, W: AsyncWrite + Unpin> LspClientRespond<'client, W> {
            #(#methods)*
        }
    };

    comment_box + "\n" + &pretty(implementation)
}
