use super::{generated::model::VERSION, DEPRECATED, PROPOSED};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn comment_box(text: &str) -> String {
    const LINES: usize = 100;

    let separator = format!("// {} //", String::from('━').repeat(LINES - 6));
    let spaces = LINES - 4 - text.len();
    let left = spaces / 2;
    let right = spaces - left;
    let line = format!(
        "//{}{text}{}//",
        String::from(' ').repeat(left),
        String::from(' ').repeat(right),
    );

    format!("{separator}\n{line}\n{separator}\n")
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn docs<'a>(
    name: impl Into<Option<&'a str>>,
    emoji: &str,
    documentation: impl Into<Option<&'a str>>,
) -> Option<TokenStream> {
    let documentation = documentation.into().unwrap_or_default();

    if documentation.contains(DEPRECATED) || documentation.contains(PROPOSED) {
        return None;
    }

    let (a, b) = if let Some(name) = name.into() {
        let link = format!(
            "{}/{VERSION}/specification/#{}",
            "https://microsoft.github.io/language-server-protocol/specifications/lsp",
            name.trim_start_matches("$/").split('/').enumerate().fold(
                String::new(),
                |mut acc, (i, name)| {
                    if i != 0 {
                        acc.push('_');
                    }
                    acc.push_str(&name.to_case(Case::Camel));
                    acc
                }
            )
        );

        assert!(!documentation.contains("[docs]"));
        (
            String::from("[📖][docs] "),
            format!("\n\n[docs]: {link} (Documentation)"),
        )
    } else {
        Default::default()
    };
    let a = a + emoji;

    Some(
        (a + documentation + &b)
            .lines()
            .map(|line| format!(" {line}"))
            .map(|line| quote! { #[doc = #line] })
            .collect(),
    )
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn ident(ident: &str) -> Ident {
    format_ident!("{ident}")
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn pascal(name: &str) -> Ident {
    let ident = ident(&name.to_case(Case::Pascal));

    if name.starts_with('_') {
        format_ident!("{ident}2")
    } else {
        ident
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn snake(name: &str) -> Ident {
    const KEYWORDS: &[&str] = &["type"];

    let ident = ident(&name.to_case(Case::Snake));

    for keyword in KEYWORDS {
        if name == *keyword {
            return format_ident!("{ident}_");
        }
    }

    ident
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub fn pretty(tokens: TokenStream) -> String {
    prettyplease::unparse(&syn::parse_file(&tokens.to_string()).unwrap())
}
