use std::ops::Index;
use virus_common::Rgba;
use virus_graphics::text::Styles;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            ThemeKey                                            //
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
    TypeParameter,
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
            "type.parameter" => Self::TypeParameter,
            "variable" => Self::Variable,
            "variable.builtin" => Self::VariableBuiltin,
            "variable.other.member" => Self::VariableOtherMember,
            "variable.parameter" => Self::VariableParameter,
            _ => Self::Default,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Theme                                              //
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
    pub type_parameter: Styles,
    pub variable: Styles,
    pub variable_builtin: Styles,
    pub variable_other_member: Styles,
    pub variable_parameter: Styles,
}

impl Theme {
    pub fn default(&self) -> &Styles {
        &self.default
    }

    /// https://github.com/catppuccin/helix/blob/main/themes/default/catppuccin_latte.toml
    pub fn catppuccin() -> Self {
        use virus_graphics::text::{
            FontStyle::{self, *},
            FontWeight::{self, *},
        };

        fn style(foreground: Rgba, weight: FontWeight, style: FontStyle) -> Styles {
            Styles {
                weight,
                style,
                underline: Default::default(),
                strike: Default::default(),
                foreground,
                background: Default::default(),
            }
        }

        let catppuccin = virus_common::Catppuccin::default();

        Self {
            default: style(catppuccin.text, Black, Normal),
            attribute: style(catppuccin.yellow, Black, Normal),
            comment: style(catppuccin.overlay2, Black, Italic),
            constant: style(catppuccin.peach, Black, Normal),
            constant_builtin_boolean: style(catppuccin.pink, Black, Normal),
            constant_character: style(catppuccin.teal, Black, Normal),
            constant_character_escape: style(catppuccin.pink, Black, Normal),
            constant_numeric_float: style(catppuccin.pink, Black, Normal),
            constant_numeric_integer: style(catppuccin.pink, Black, Normal),
            constructor: style(catppuccin.sapphire, Black, Normal),
            function: style(catppuccin.blue, Black, Normal),
            function_macro: style(catppuccin.mauve, Black, Normal),
            function_method: style(catppuccin.mauve, Black, Normal),
            keyword: style(catppuccin.mauve, Black, Normal),
            keyword_control: style(catppuccin.mauve, Black, Normal),
            keyword_control_conditional: style(catppuccin.mauve, Black, Normal),
            keyword_control_import: style(catppuccin.mauve, Black, Normal),
            keyword_control_repeat: style(catppuccin.mauve, Black, Normal),
            keyword_control_return: style(catppuccin.mauve, Black, Normal),
            keyword_function: style(catppuccin.mauve, Black, Normal),
            keyword_operator: style(catppuccin.mauve, Black, Normal),
            keyword_special: style(catppuccin.mauve, Black, Normal),
            keyword_storage: style(catppuccin.mauve, Black, Normal),
            keyword_storage_modifier: style(catppuccin.mauve, Black, Normal),
            keyword_storage_modifier_mut: style(catppuccin.mauve, Black, Normal),
            keyword_storage_modifier_ref: style(catppuccin.mauve, Black, Normal),
            keyword_storage_type: style(catppuccin.mauve, Black, Normal),
            label: style(catppuccin.sapphire, Black, Normal),
            namespace: style(catppuccin.yellow, Black, Normal),
            operator: style(catppuccin.sky, Black, Normal),
            punctuation_bracket: style(catppuccin.overlay2, Black, Normal),
            punctuation_delimiter: style(catppuccin.sky, Black, Normal),
            special: style(catppuccin.blue, Black, Normal),
            string: style(catppuccin.green, Black, Normal),
            r#type: style(catppuccin.yellow, Black, Normal),
            type_builtin: style(catppuccin.yellow, Black, Normal),
            type_enum_variant: style(catppuccin.teal, Black, Normal),
            type_parameter: style(catppuccin.yellow, Black, Normal),
            variable: style(catppuccin.text, Black, Normal),
            variable_builtin: style(catppuccin.red, Black, Normal),
            variable_other_member: style(catppuccin.teal, Black, Normal),
            variable_parameter: style(catppuccin.maroon, Black, Normal),
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
            ThemeKey::TypeParameter => &self.type_parameter,
            ThemeKey::Variable => &self.variable,
            ThemeKey::VariableBuiltin => &self.variable_builtin,
            ThemeKey::VariableOtherMember => &self.variable_other_member,
            ThemeKey::VariableParameter => &self.variable_parameter,
        }
    }
}
