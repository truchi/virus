use super::super::{
    generated::schema::{MessageDirection, Notification, Type},
    utils::{docs, pascal},
    Model, Quote,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

impl<'a> Quote<&'a Notification> {
    pub fn quote(
        self,
        model: &Model,
    ) -> Option<(/* pascal */ Ident, /* types */ Vec<TokenStream>)> {
        if self.0.deprecated.is_some() || self.0.proposed.is_some() {
            return None;
        }

        let method = &self.0.method;
        let documentation = docs(
            method.as_ref(),
            match self.0.message_direction {
                MessageDirection::ClientToServer => "➡️ ",
                MessageDirection::ServerToClient => "⬅️ ",
                MessageDirection::Both => "➡️ ⬅️ ",
            },
            self.0.documentation.as_deref(),
        )?;
        let pascal_method = pascal(&method.trim_start_matches("$/").replace('/', "_"));
        let declarations_and_usage = |type_: Option<&Type>, name: &str| {
            if let Some(type_) = type_ {
                let (declaration, dependencies, usage, _) = Quote(type_)
                    .quote(&format!("{pascal_method}{name}"), model)
                    .unwrap();
                let declarations = declaration.into_iter().chain(dependencies);

                (quote! { #(#declarations)* }, usage)
            } else {
                (quote! {}, quote! { () })
            }
        };
        let types = {
            let registration_method = self
                .0
                .registration_method
                .as_ref()
                .map(|method| quote! { Some(#method) })
                .unwrap_or_else(|| quote! { None });
            let (registration_options_declarations, registration_options) =
                declarations_and_usage(self.0.registration_options.as_ref(), "RegistrationOptions");
            let (params_declarations, params) = declarations_and_usage(
                self.0.params.as_ref().and_then(|params| {
                    assert!(params.subtype_1.is_none());
                    params.subtype_0.as_ref()
                }),
                "Params",
            );

            vec![
                quote! {
                    #documentation
                    pub enum #pascal_method {}
                },
                quote! {
                    impl NotificationTrait for #pascal_method {
                        const REGISTRATION_METHOD: Option<&'static str> = #registration_method;
                        const METHOD: &'static str = #method;
                        type RegistrationOptions = #registration_options;
                        type Params = #params;
                    }
                },
                registration_options_declarations,
                params_declarations,
            ]
        };

        Some((pascal_method, types))
    }
}
