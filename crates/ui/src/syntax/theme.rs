use std::ops::Index;
use virus_editor::ast::HighlightsTag;
use virus_graphics::text::Styles;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          SyntaxTheme                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SyntaxTheme {
    pub default: Styles,
    pub attribute: Styles,
    pub comment: Styles,
    pub constant: Styles,
    pub constant_builtin_boolean: Styles,
    pub constant_character: Styles,
    pub constant_character_escape: Styles,
    pub constant_numeric_float: Styles,
    pub constant_numeric_integer: Styles,
    pub constructor: Styles,
    pub function: Styles,
    pub function_macro: Styles,
    pub function_method: Styles,
    pub keyword: Styles,
    pub keyword_control: Styles,
    pub keyword_control_conditional: Styles,
    pub keyword_control_import: Styles,
    pub keyword_control_repeat: Styles,
    pub keyword_control_return: Styles,
    pub keyword_function: Styles,
    pub keyword_operator: Styles,
    pub keyword_special: Styles,
    pub keyword_storage: Styles,
    pub keyword_storage_modifier: Styles,
    pub keyword_storage_modifier_mut: Styles,
    pub keyword_storage_modifier_ref: Styles,
    pub keyword_storage_type: Styles,
    pub label: Styles,
    pub namespace: Styles,
    pub operator: Styles,
    pub punctuation_bracket: Styles,
    pub punctuation_delimiter: Styles,
    pub special: Styles,
    pub string: Styles,
    pub r#type: Styles,
    pub type_builtin: Styles,
    pub type_enum_variant: Styles,
    pub type_parameter: Styles,
    pub variable: Styles,
    pub variable_builtin: Styles,
    pub variable_other_member: Styles,
    pub variable_parameter: Styles,
}

impl Index<HighlightsTag> for SyntaxTheme {
    type Output = Styles;

    fn index(&self, tag: HighlightsTag) -> &Self::Output {
        match tag {
            HighlightsTag::Attribute => &self.attribute,
            HighlightsTag::Comment => &self.comment,
            HighlightsTag::Constant => &self.constant,
            HighlightsTag::ConstantBuiltinBoolean => &self.constant_builtin_boolean,
            HighlightsTag::ConstantCharacter => &self.constant_character,
            HighlightsTag::ConstantCharacterEscape => &self.constant_character_escape,
            HighlightsTag::ConstantNumericFloat => &self.constant_numeric_float,
            HighlightsTag::ConstantNumericInteger => &self.constant_numeric_integer,
            HighlightsTag::Constructor => &self.constructor,
            HighlightsTag::Function => &self.function,
            HighlightsTag::FunctionMacro => &self.function_macro,
            HighlightsTag::FunctionMethod => &self.function_method,
            HighlightsTag::Keyword => &self.keyword,
            HighlightsTag::KeywordControl => &self.keyword_control,
            HighlightsTag::KeywordControlConditional => &self.keyword_control_conditional,
            HighlightsTag::KeywordControlImport => &self.keyword_control_import,
            HighlightsTag::KeywordControlRepeat => &self.keyword_control_repeat,
            HighlightsTag::KeywordControlReturn => &self.keyword_control_return,
            HighlightsTag::KeywordFunction => &self.keyword_function,
            HighlightsTag::KeywordOperator => &self.keyword_operator,
            HighlightsTag::KeywordSpecial => &self.keyword_special,
            HighlightsTag::KeywordStorage => &self.keyword_storage,
            HighlightsTag::KeywordStorageModifier => &self.keyword_storage_modifier,
            HighlightsTag::KeywordStorageModifierMut => &self.keyword_storage_modifier_mut,
            HighlightsTag::KeywordStorageModifierRef => &self.keyword_storage_modifier_ref,
            HighlightsTag::KeywordStorageType => &self.keyword_storage_type,
            HighlightsTag::Label => &self.label,
            HighlightsTag::Namespace => &self.namespace,
            HighlightsTag::Operator => &self.operator,
            HighlightsTag::PunctuationBracket => &self.punctuation_bracket,
            HighlightsTag::PunctuationDelimiter => &self.punctuation_delimiter,
            HighlightsTag::Special => &self.special,
            HighlightsTag::String => &self.string,
            HighlightsTag::Type => &self.r#type,
            HighlightsTag::TypeBuiltin => &self.type_builtin,
            HighlightsTag::TypeEnumVariant => &self.type_enum_variant,
            HighlightsTag::TypeParameter => &self.type_parameter,
            HighlightsTag::Variable => &self.variable,
            HighlightsTag::VariableBuiltin => &self.variable_builtin,
            HighlightsTag::VariableOtherMember => &self.variable_other_member,
            HighlightsTag::VariableParameter => &self.variable_parameter,
        }
    }
}

impl Index<Option<HighlightsTag>> for SyntaxTheme {
    type Output = Styles;

    fn index(&self, tag: Option<HighlightsTag>) -> &Self::Output {
        match tag {
            Some(tag) => &self[tag],
            None => &self.default,
        }
    }
}
