use virus_common::{Rgba, Style};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Theme                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Theme {
    pub default: Style,
    pub attribute: Style,
    pub comment: Style,
    pub constant: Style,
    pub constant_builtin_boolean: Style,
    pub constant_character: Style,
    pub constant_character_escape: Style,
    pub constant_numeric_float: Style,
    pub constant_numeric_integer: Style,
    pub constructor: Style,
    pub function: Style,
    pub function_macro: Style,
    pub function_method: Style,
    pub keyword: Style,
    pub keyword_control: Style,
    pub keyword_control_conditional: Style,
    pub keyword_control_import: Style,
    pub keyword_control_repeat: Style,
    pub keyword_control_return: Style,
    pub keyword_function: Style,
    pub keyword_operator: Style,
    pub keyword_special: Style,
    pub keyword_storage: Style,
    pub keyword_storage_modifier: Style,
    pub keyword_storage_modifier_mut: Style,
    pub keyword_storage_modifier_ref: Style,
    pub keyword_storage_type: Style,
    pub label: Style,
    pub namespace: Style,
    pub operator: Style,
    pub punctuation_bracket: Style,
    pub punctuation_delimiter: Style,
    pub special: Style,
    pub string: Style,
    pub r#type: Style,
    pub type_builtin: Style,
    pub type_enum_variant: Style,
    pub variable: Style,
    pub variable_builtin: Style,
    pub variable_other_member: Style,
    pub variable_parameter: Style,
}

impl Theme {
    pub fn default(&self) -> &Style {
        &self.default
    }

    pub fn get(&self, key: &str) -> &Style {
        match key {
            "attribute" => &self.attribute,
            "comment" => &self.comment,
            "constant" => &self.constant,
            "constant.builtin.boolean" => &self.constant_builtin_boolean,
            "constant.character" => &self.constant_character,
            "constant.character.escape" => &self.constant_character_escape,
            "constant.numeric.float" => &self.constant_numeric_float,
            "constant.numeric.integer" => &self.constant_numeric_integer,
            "constructor" => &self.constructor,
            "function" => &self.function,
            "function.macro" => &self.function_macro,
            "function.method" => &self.function_method,
            "keyword" => &self.keyword,
            "keyword.control" => &self.keyword_control,
            "keyword.control.conditional" => &self.keyword_control_conditional,
            "keyword.control.import" => &self.keyword_control_import,
            "keyword.control.repeat" => &self.keyword_control_repeat,
            "keyword.control.return" => &self.keyword_control_return,
            "keyword.function" => &self.keyword_function,
            "keyword.operator" => &self.keyword_operator,
            "keyword.special" => &self.keyword_special,
            "keyword.storage" => &self.keyword_storage,
            "keyword.storage.modifier" => &self.keyword_storage_modifier,
            "keyword.storage.modifier.mut" => &self.keyword_storage_modifier_mut,
            "keyword.storage.modifier.ref" => &self.keyword_storage_modifier_ref,
            "keyword.storage.type" => &self.keyword_storage_type,
            "label" => &self.label,
            "namespace" => &self.namespace,
            "operator" => &self.operator,
            "punctuation.bracket" => &self.punctuation_bracket,
            "punctuation.delimiter" => &self.punctuation_delimiter,
            "special" => &self.special,
            "string" => &self.string,
            "type" => &self.r#type,
            "type.builtin" => &self.type_builtin,
            "type.enum.variant" => &self.type_enum_variant,
            "variable" => &self.variable,
            "variable.builtin" => &self.variable_builtin,
            "variable.other.member" => &self.variable_other_member,
            "variable.parameter" => &self.variable_parameter,
            _ => &self.default,
        }
    }

    pub fn dracula() -> Self {
        fn style(r: u8, g: u8, b: u8) -> Style {
            Style {
                foreground: Rgba {
                    r,
                    g,
                    b,
                    a: u8::MAX,
                },
                ..Default::default()
            }
        }

        let _background = style(40, 42, 54);
        let _current = style(68, 71, 90);
        let foreground = style(248, 248, 242);
        let comment = style(98, 114, 164);
        let cyan = style(139, 233, 253);
        let green = style(80, 250, 123);
        let orange = style(255, 184, 108);
        let pink = style(255, 121, 198);
        let purple = style(189, 147, 249);
        let red = style(255, 85, 85);
        let yellow = style(241, 250, 140);

        Theme {
            default: style(255, 255, 255),
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
