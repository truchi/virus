use std::str::FromStr;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub const RUST_HIGHLIGHTS: &'static str = include_str!("./queries/rust/highlights.scm");

// ────────────────────────────────────────────────────────────────────────────────────────────── //

macro_rules! tag {
    ($(#[$meta:meta])* $vis:vis $Enum:ident, [$(($Variant:ident, $str:literal)),* $(,)?]) => {
        $(#[$meta])*
        $vis enum $Enum { $(
            $Variant,
        )* }

        impl FromStr for $Enum {
            type Err = ();

            fn from_str(str: &str) -> Result<Self, Self::Err> {
                Ok(match str {
                    $($str => Self::$Variant,)*
                    _ => return Err(()),
                })
            }
        }

        impl TryFrom<u8> for $Enum {
            type Error = ();

            fn try_from(u8: u8) -> Result<Self, Self::Error> {
                [$(Self::$Variant,)*]
                .get(u8 as usize)
                .copied()
                .ok_or(())
            }
        }
    };
}

tag!(
    /// Tags found in `highlights.scm` files.
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub HighlightsTag,
    [
        (Attribute, "attribute"),
        (Comment, "comment"),
        (Constant, "constant"),
        (ConstantBuiltinBoolean, "constant.builtin.boolean"),
        (ConstantCharacter, "constant.character"),
        (ConstantCharacterEscape, "constant.character.escape"),
        (ConstantNumericFloat, "constant.numeric.float"),
        (ConstantNumericInteger, "constant.numeric.integer"),
        (Constructor, "constructor"),
        (Function, "function"),
        (FunctionMacro, "function.macro"),
        (FunctionMethod, "function.method"),
        (Keyword, "keyword"),
        (KeywordControl, "keyword.control"),
        (KeywordControlConditional, "keyword.control.conditional"),
        (KeywordControlImport, "keyword.control.import"),
        (KeywordControlRepeat, "keyword.control.repeat"),
        (KeywordControlReturn, "keyword.control.return"),
        (KeywordFunction, "keyword.function"),
        (KeywordOperator, "keyword.operator"),
        (KeywordSpecial, "keyword.special"),
        (KeywordStorage, "keyword.storage"),
        (KeywordStorageModifier, "keyword.storage.modifier"),
        (KeywordStorageModifierMut, "keyword.storage.modifier.mut"),
        (KeywordStorageModifierRef, "keyword.storage.modifier.ref"),
        (KeywordStorageType, "keyword.storage.type"),
        (Label, "label"),
        (Namespace, "namespace"),
        (Operator, "operator"),
        (PunctuationBracket, "punctuation.bracket"),
        (PunctuationDelimiter, "punctuation.delimiter"),
        (Special, "special"),
        (String, "string"),
        (Type, "type"),
        (TypeBuiltin, "type.builtin"),
        (TypeEnumVariant, "type.enum.variant"),
        (TypeParameter, "type.parameter"),
        (Variable, "variable"),
        (VariableBuiltin, "variable.builtin"),
        (VariableOtherMember, "variable.other.member"),
        (VariableParameter, "variable.parameter"),
    ]
);

#[cfg(test)]
mod tests {
    // TODO write a test that checks tags in all highlights.scm (and future other .scm files)
}
