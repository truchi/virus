use super::super::{
    generated::schema::Structure,
    utils::{comment_box, docs, pascal, pretty, snake},
    Model, Quote,
};
use quote::quote;

impl<'a> Quote<&'a Structure> {
    pub fn quote(self, model: &Model) -> Option<String> {
        if self.0.deprecated.is_some() || self.0.proposed.is_some() {
            return None;
        }

        let structure_name = pascal(&self.0.name);
        let comment_box = comment_box(&structure_name.to_string());
        let documentation = docs(self.0.name.as_ref(), "", self.0.documentation.as_deref())?;
        let extends = self
            .0
            .extends(&model.model.structures)
            .flat_map(|structure| {
                if structure.deprecated.is_some() || structure.proposed.is_some() {
                    return None;
                }

                let documentation = docs(None, "", structure.documentation.as_deref())?;
                let snake_name = snake(&structure.name);
                let pascal_name = pascal(&structure.name);

                Some(quote! {
                    #documentation
                    #[serde(flatten)]
                    pub #snake_name: #pascal_name,
                })
            });
        let mut all_dependencies = Vec::new();
        let properties = self.0.properties.iter().flat_map(|property| {
            if property.deprecated.is_some() || property.proposed.is_some() {
                return None;
            }

            let documentation = docs(None, "", property.documentation.as_deref());
            let name = &property.name;
            let snake_name = snake(name);
            let pascal_name = pascal(name);

            let (declaration, dependencies, type_, _) =
                Quote(&property.type_).quote(&format!("{structure_name}{pascal_name}"), model)?;
            all_dependencies.extend(declaration.into_iter().chain(dependencies));

            let type_ = (type_.to_string().split("::").last().unwrap().trim()
                == structure_name.to_string())
            .then(|| quote! { Box<#type_> })
            .unwrap_or(type_);
            let type_ = (property.optional == Some(true))
                .then(|| quote! { Option<#type_> })
                .unwrap_or(type_);

            Some(quote! {
                #documentation
                #[serde(rename = #name)]
                pub #snake_name: #type_,
            })
        });

        let structure = pretty(quote! {
            #documentation
            #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
            pub struct #structure_name {
                #(#extends)*
                #(#properties)*
            }
        });
        let dependencies = all_dependencies
            .into_iter()
            .map(pretty)
            .map(|dependency| dependency + "\n")
            .collect::<String>();

        Some(format!("{comment_box}\n{structure}\n{dependencies}\n"))
    }
}
