use std::ops::Index;
use virus_common::Rgba;
use virus_graphics::text::Styles;

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
    pub fn catppuccin_latte() -> Self {
        use virus_graphics::text::{
            FontStyle::{self, *},
            FontWeight::{self, *},
        };

        fn style(color: &str, weight: FontWeight, style: FontStyle) -> Styles {
            let (r, g, b) = (
                u8::from_str_radix(&color[1..3], 16).unwrap(),
                u8::from_str_radix(&color[3..5], 16).unwrap(),
                u8::from_str_radix(&color[5..7], 16).unwrap(),
            );

            Styles {
                weight,
                style,
                underline: Default::default(),
                strike: Default::default(),
                foreground: Rgba::new(r, g, b, u8::MAX),
                background: Default::default(),
            }
        }

        let _rosewater = "#dc8a78";
        let _flamingo = "#dd7878";
        let pink = "#ea76cb";
        let mauve = "#8839ef";
        let red = "#d20f39";
        let maroon = "#e64553";
        let peach = "#fe640b";
        let yellow = "#df8e1d";
        let green = "#40a02b";
        let teal = "#179299";
        let sky = "#04a5e5";
        let sapphire = "#209fb5";
        let blue = "#1e66f5";
        let _lavender = "#7287fd";
        let text = "#4c4f69";
        let _subtext1 = "#5c5f77";
        let _subtext0 = "#6c6f85";
        let overlay2 = "#7c7f93";
        let _overlay1 = "#8c8fa1";
        let _overlay0 = "#9ca0b0";
        let _surface2 = "#acb0be";
        let _surface1 = "#bcc0cc";
        let _surface0 = "#ccd0da";
        let _base = "#eff1f5";
        let _mantle = "#e6e9ef";
        let _crust = "#dce0e8";
        let regular = Black;

        Self {
            default: style(text, regular, Normal),
            attribute: style(yellow, regular, Normal),
            comment: style(overlay2, regular, Italic),
            constant: style(peach, regular, Normal),
            constant_builtin_boolean: style(pink, regular, Normal),
            constant_character: style(teal, regular, Normal),
            constant_character_escape: style(pink, regular, Normal),
            constant_numeric_float: style(pink, regular, Normal),
            constant_numeric_integer: style(pink, regular, Normal),
            constructor: style(sapphire, regular, Normal),
            function: style(blue, regular, Normal),
            function_macro: style(mauve, regular, Normal),
            function_method: style(mauve, regular, Normal),
            keyword: style(mauve, regular, Normal),
            keyword_control: style(mauve, regular, Normal),
            keyword_control_conditional: style(mauve, regular, Normal),
            keyword_control_import: style(mauve, regular, Normal),
            keyword_control_repeat: style(mauve, regular, Normal),
            keyword_control_return: style(mauve, regular, Normal),
            keyword_function: style(mauve, regular, Normal),
            keyword_operator: style(mauve, regular, Normal),
            keyword_special: style(mauve, regular, Normal),
            keyword_storage: style(mauve, regular, Normal),
            keyword_storage_modifier: style(mauve, regular, Normal),
            keyword_storage_modifier_mut: style(mauve, regular, Normal),
            keyword_storage_modifier_ref: style(mauve, regular, Normal),
            keyword_storage_type: style(mauve, regular, Normal),
            label: style(sapphire, regular, Normal),
            namespace: style(yellow, regular, Normal),
            operator: style(sky, regular, Normal),
            punctuation_bracket: style(overlay2, regular, Normal),
            punctuation_delimiter: style(sky, regular, Normal),
            special: style(blue, regular, Normal),
            string: style(green, regular, Normal),
            r#type: style(yellow, regular, Normal),
            type_builtin: style(yellow, regular, Normal),
            type_enum_variant: style(teal, regular, Normal),
            type_parameter: style(yellow, regular, Normal),
            variable: style(text, regular, Normal),
            variable_builtin: style(red, regular, Normal),
            variable_other_member: style(teal, regular, Normal),
            variable_parameter: style(maroon, regular, Normal),
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
