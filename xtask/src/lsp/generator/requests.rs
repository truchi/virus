use super::super::{
    generated::schema::{AndType, BaseType, BaseTypes, MessageDirection, Request, Type},
    utils::{docs, pascal},
    Model, Quote,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

impl<'a> Quote<&'a Request> {
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
                MessageDirection::ClientToServer => "↩️ ",
                MessageDirection::ServerToClient => "↪️ ",
                MessageDirection::Both => "↩️ ↪️ ",
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
            let (partial_result_declarations, partial_result) =
                declarations_and_usage(self.0.partial_result.as_ref(), "PartialResult");
            let (result_declarations, result) =
                declarations_and_usage(Some(&self.0.result), "Result");
            let (error_declarations, error) =
                declarations_and_usage(self.0.error_data.as_ref(), "Error");
            let params_fn = if self.0.params.is_some() {
                quote! { Some(params) }
            } else {
                quote! {
                    let _ = params;
                    None
                }
            };
            let error_fn = if self.0.error_data.is_some() {
                quote! { error }
            } else {
                quote! {
                    debug_assert!(error.data.is_none());
                    Error::new(error.code, error.message, None)
                }
            };
            let deserialize_request = if self.0.params.is_some() {
                quote! {
                    Ok(serde_json::from_value(request.params.ok_or_else(missing_params)?)?)
                }
            } else {
                quote! {
                    debug_assert!(request.params.is_none());
                    Ok(())
                }
            };
            let deserialize_response = {
                let ok_default = match &self.0.result {
                    Type::AndType(AndType { items, kind })
                        if kind == "or"
                            && items
                                .iter()
                                .filter_map(|item| match item {
                                    Type::BaseType(BaseType { name, .. }) => Some(name),
                                    _ => None,
                                })
                                .find(|name| **name == BaseTypes::Null)
                                .is_some() =>
                    {
                        Some(quote! { #result::Null(Null) })
                    }
                    Type::BaseType(BaseType { name, .. }) if *name == BaseTypes::Null => {
                        Some(quote! { Null })
                    }
                    // NOTE we could also check through type aliases...
                    _ => None,
                };
                let ok = if let Some(ok_default) = ok_default {
                    quote! {
                        response
                            .result
                            .map(|result| serde_json::from_value(result))
                            .transpose()?
                            .unwrap_or(#ok_default)
                    }
                } else {
                    quote! {
                        serde_json::from_value(response.result.ok_or_else(missing_result)?)?
                    }
                };
                let err = if self.0.error_data.is_some() {
                    quote! {{
                        debug_assert!(response.result.is_none());
                        Error::new(
                            error.code,
                            error.message,
                            error.data.map(serde_json::from_value).transpose()?,
                        )
                    }}
                } else {
                    quote! {{
                        debug_assert!(response.result.is_none());
                        debug_assert!(error.data.is_none());
                        Error::new(error.code, error.message, None)
                    }}
                };

                quote! {
                    Ok(if let Some(error) = response.error {
                        Err(#err)
                    } else {
                        Ok(#ok)
                    })
                }
            };

            vec![
                quote! {
                    #documentation
                    pub enum #pascal_method {}
                },
                quote! {
                    impl RequestTrait for #pascal_method {
                        const REGISTRATION_METHOD: Option<&'static str> = #registration_method;
                        const METHOD: &'static str = #method;
                        type RegistrationOptions = #registration_options;
                        type Params = #params;
                        type PartialResult = #partial_result;
                        type Result = #result;
                        type Error = #error;

                        fn params(params: Self::Params) -> Option<Self::Params> {
                            #params_fn
                        }

                        fn error(error: Error<Self::Error>) -> Error<Self::Error> {
                            #error_fn
                        }

                        fn deserialize_request(request: super::Request<Value>) -> std::io::Result<Self::Params> {
                            debug_assert!(request.method == #method);
                            #deserialize_request
                        }

                        fn deserialize_response(response: super::Response<Value, Value>) -> std::io::Result<Result<Self::Result, Error<Self::Error>>> {
                            #deserialize_response
                        }
                    }
                },
                registration_options_declarations,
                params_declarations,
                partial_result_declarations,
                result_declarations,
                error_declarations,
            ]
        };

        Some((pascal_method, types))
    }
}
