use std::ops::Index;
use virus_graphics::{colors::Rgba, text::Styles};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             ThemeKey                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum ThemeKey {
    #[default]
    Default,
    Attribute,
    Comment,
    Constant,
    ConstantBuiltinBoolean,
    ConstantCharacter,
    ConstantCharacterEscape,
    ConstantNumericFloat,
    ConstantNumericInteger,
    Constructor,
    Function,
    FunctionMacro,
    FunctionMethod,
    Keyword,
    KeywordControl,
    KeywordControlConditional,
    KeywordControlImport,
    KeywordControlRepeat,
    KeywordControlReturn,
    KeywordFunction,
    KeywordOperator,
    KeywordSpecial,
    KeywordStorage,
    KeywordStorageModifier,
    KeywordStorageModifierMut,
    KeywordStorageModifierRef,
    KeywordStorageType,
    Label,
    Namespace,
    Operator,
    PunctuationBracket,
    PunctuationDelimiter,
    Special,
    String,
    Type,
    TypeBuiltin,
    TypeEnumVariant,
    Variable,
    VariableBuiltin,
    VariableOtherMember,
    VariableParameter,
}

impl ThemeKey {
    pub fn new(str: &str) -> Self {
        match str {
            "attribute" => Self::Attribute,
            "comment" => Self::Comment,
            "constant" => Self::Constant,
            "constant.builtin.boolean" => Self::ConstantBuiltinBoolean,
            "constant.character" => Self::ConstantCharacter,
            "constant.character.escape" => Self::ConstantCharacterEscape,
            "constant.numeric.float" => Self::ConstantNumericFloat,
            "constant.numeric.integer" => Self::ConstantNumericInteger,
            "constructor" => Self::Constructor,
            "function" => Self::Function,
            "function.macro" => Self::FunctionMacro,
            "function.method" => Self::FunctionMethod,
            "keyword" => Self::Keyword,
            "keyword.control" => Self::KeywordControl,
            "keyword.control.conditional" => Self::KeywordControlConditional,
            "keyword.control.import" => Self::KeywordControlImport,
            "keyword.control.repeat" => Self::KeywordControlRepeat,
            "keyword.control.return" => Self::KeywordControlReturn,
            "keyword.function" => Self::KeywordFunction,
            "keyword.operator" => Self::KeywordOperator,
            "keyword.special" => Self::KeywordSpecial,
            "keyword.storage" => Self::KeywordStorage,
            "keyword.storage.modifier" => Self::KeywordStorageModifier,
            "keyword.storage.modifier.mut" => Self::KeywordStorageModifierMut,
            "keyword.storage.modifier.ref" => Self::KeywordStorageModifierRef,
            "keyword.storage.type" => Self::KeywordStorageType,
            "label" => Self::Label,
            "namespace" => Self::Namespace,
            "operator" => Self::Operator,
            "punctuation.bracket" => Self::PunctuationBracket,
            "punctuation.delimiter" => Self::PunctuationDelimiter,
            "special" => Self::Special,
            "string" => Self::String,
            "type" => Self::Type,
            "type.builtin" => Self::TypeBuiltin,
            "type.enum.variant" => Self::TypeEnumVariant,
            "variable" => Self::Variable,
            "variable.builtin" => Self::VariableBuiltin,
            "variable.other.member" => Self::VariableOtherMember,
            "variable.parameter" => Self::VariableParameter,
            _ => Self::Default,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Theme                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Theme {
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
    pub variable: Styles,
    pub variable_builtin: Styles,
    pub variable_other_member: Styles,
    pub variable_parameter: Styles,
}

impl Theme {
    pub fn default(&self) -> &Styles {
        &self.default
    }

    /// Stupid shit for tests!
    pub fn dracula() -> Self {
        use virus_graphics::text::{
            FontStyle::{self, *},
            FontWeight::{self, *},
            Shadow,
        };

        fn style(r: u8, g: u8, b: u8, weight: FontWeight, style: FontStyle) -> Styles {
            Styles {
                weight,
                style,
                underline: Default::default(),
                strike: Default::default(),
                foreground: Rgba::new(r, g, b, u8::MAX),
                background: Rgba::TRANSPARENT,
                shadow: Some(Shadow {
                    radius: 10,
                    color: Rgba::new(b, r, g, u8::MAX),
                }),
            }
        }

        let _background = style(40, 42, 54, Regular, Normal);
        let _current = style(68, 71, 90, Regular, Normal);
        let foreground = style(248, 248, 242, Regular, Normal);
        let comment = style(98, 114, 164, Regular, Italic);
        let cyan = style(139, 233, 253, Bold, Normal);
        let green = style(80, 250, 123, Bold, Normal);
        let orange = style(255, 184, 108, Regular, Normal);
        let pink = style(255, 121, 198, Regular, Oblique);
        let purple = style(189, 147, 249, Regular, Normal);
        let red = style(255, 85, 85, Regular, Normal);
        let yellow = style(241, 250, 140, Regular, Normal);

        Theme {
            default: style(255, 255, 255, Regular, Normal),
            attribute: green,
            comment,
            constant: green,
            constant_builtin_boolean: purple,
            constant_character: purple,
            constant_character_escape: purple,
            constant_numeric_float: purple,
            constant_numeric_integer: purple,
            constructor: foreground,
            function: pink,
            function_macro: pink,
            function_method: pink,
            keyword: red,
            keyword_control: red,
            keyword_control_conditional: red,
            keyword_control_import: red,
            keyword_control_repeat: red,
            keyword_control_return: red,
            keyword_function: red,
            keyword_operator: red,
            keyword_special: red,
            keyword_storage: red,
            keyword_storage_modifier: red,
            keyword_storage_modifier_mut: red,
            keyword_storage_modifier_ref: red,
            keyword_storage_type: red,
            label: foreground,
            namespace: foreground,
            operator: foreground,
            punctuation_bracket: yellow,
            punctuation_delimiter: yellow,
            special: yellow,
            string: cyan,
            r#type: cyan,
            type_builtin: cyan,
            type_enum_variant: cyan,
            variable: orange,
            variable_builtin: orange,
            variable_other_member: orange,
            variable_parameter: orange,
        }
    }
}

impl Index<ThemeKey> for Theme {
    type Output = Styles;

    fn index(&self, key: ThemeKey) -> &Self::Output {
        match key {
            ThemeKey::Default => &self.default,
            ThemeKey::Attribute => &self.attribute,
            ThemeKey::Comment => &self.comment,
            ThemeKey::Constant => &self.constant,
            ThemeKey::ConstantBuiltinBoolean => &self.constant_builtin_boolean,
            ThemeKey::ConstantCharacter => &self.constant_character,
            ThemeKey::ConstantCharacterEscape => &self.constant_character_escape,
            ThemeKey::ConstantNumericFloat => &self.constant_numeric_float,
            ThemeKey::ConstantNumericInteger => &self.constant_numeric_integer,
            ThemeKey::Constructor => &self.constructor,
            ThemeKey::Function => &self.function,
            ThemeKey::FunctionMacro => &self.function_macro,
            ThemeKey::FunctionMethod => &self.function_method,
            ThemeKey::Keyword => &self.keyword,
            ThemeKey::KeywordControl => &self.keyword_control,
            ThemeKey::KeywordControlConditional => &self.keyword_control_conditional,
            ThemeKey::KeywordControlImport => &self.keyword_control_import,
            ThemeKey::KeywordControlRepeat => &self.keyword_control_repeat,
            ThemeKey::KeywordControlReturn => &self.keyword_control_return,
            ThemeKey::KeywordFunction => &self.keyword_function,
            ThemeKey::KeywordOperator => &self.keyword_operator,
            ThemeKey::KeywordSpecial => &self.keyword_special,
            ThemeKey::KeywordStorage => &self.keyword_storage,
            ThemeKey::KeywordStorageModifier => &self.keyword_storage_modifier,
            ThemeKey::KeywordStorageModifierMut => &self.keyword_storage_modifier_mut,
            ThemeKey::KeywordStorageModifierRef => &self.keyword_storage_modifier_ref,
            ThemeKey::KeywordStorageType => &self.keyword_storage_type,
            ThemeKey::Label => &self.label,
            ThemeKey::Namespace => &self.namespace,
            ThemeKey::Operator => &self.operator,
            ThemeKey::PunctuationBracket => &self.punctuation_bracket,
            ThemeKey::PunctuationDelimiter => &self.punctuation_delimiter,
            ThemeKey::Special => &self.special,
            ThemeKey::String => &self.string,
            ThemeKey::Type => &self.r#type,
            ThemeKey::TypeBuiltin => &self.type_builtin,
            ThemeKey::TypeEnumVariant => &self.type_enum_variant,
            ThemeKey::Variable => &self.variable,
            ThemeKey::VariableBuiltin => &self.variable_builtin,
            ThemeKey::VariableOtherMember => &self.variable_other_member,
            ThemeKey::VariableParameter => &self.variable_parameter,
        }
    }
}
