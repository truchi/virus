use super::super::{
    generated::schema::TypeAlias,
    utils::{comment_box, docs, pascal, pretty},
    Model, Quote,
};
use quote::quote;

impl<'a> Quote<&'a TypeAlias> {
    pub fn quote(self, model: &Model) -> Option<String> {
        if self.0.deprecated.is_some() || self.0.proposed.is_some() {
            return None;
        }

        let type_alias_name = pascal(&self.0.name);
        let comment_box = comment_box(&type_alias_name.to_string());
        let documentation = docs(self.0.name.as_ref(), "", self.0.documentation.as_deref());
        let (declaration, dependencies, type_, _) =
            Quote(&self.0.type_).quote(&self.0.name, model)?;
        let declaration = if let Some(declaration) = declaration {
            quote! { #declaration }
        } else {
            quote! { pub type #type_alias_name = #type_; }
        };

        let type_alias = pretty(quote! {
            #documentation
            #declaration
        });
        let dependencies = dependencies
            .into_iter()
            .map(pretty)
            .map(|dependency| dependency + "\n")
            .collect::<String>();

        Some(format!("{comment_box}\n{type_alias}\n{dependencies}\n"))
    }
}
