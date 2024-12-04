use crate::events::{Event, Key, KeyEvent, Mods};
use serde::Deserialize;
use smol_str::{SmolStr, ToSmolStr};
use std::{
    collections::{HashMap, HashSet},
    iter::{Filter, Peekable},
    slice::Iter,
    str::Split,
};
use virus_editor::mode::Mode;

const UNSTICK: &'static str = "unstick";

const MISSING_TOKEN: &'static str = "Missing token";
const UNEXPECTED_TOKEN: &'static str = "Unexpected token";
const UNKNOWN_ACTION: &'static str = "Unknown action";
const UNKNOWN_ALIAS: &'static str = "Unknown alias";
const INVALID_IDENTIFIER: &'static str = "Invalid identifier";
const INVALID_VALUE: &'static str = "Invalid value";
const INVALID_TOKEN: &'static str = "Invalid token";
const UNKNOWN_ARGUMENT: &'static str = "Unknown argument";
const INVALID_ARGUMENT: &'static str = "Invalid argument";
const CONFLICTING_BINDING: &'static str = "Conflicting binding";
const UNEXPECTED_UNSTICK: &'static str = "Unexpected unstick";

type NodeId = usize;
type Number = usize;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        KeybindingsError                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`KeybindingsError`] result.
type KeybindingsResult<T> = Result<T, KeybindingsError>;

use KeybindingsError::*;

/// [`Keybindings`] errors.
#[derive(thiserror::Error, Debug)]
pub enum KeybindingsError {
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Invalid identifier `{identifier}`")]
    InvalidIdentifier { identifier: SmolStr },

    #[error("Missing token(s) in alias `{alias}`")]
    MissingTokenInAlias { alias: SmolStr },

    // NOTE: It could be nice to have alias recursion resolution!
    #[error("Unimplemented recursion (`{token}`) in alias `{alias}`")]
    UnimplementedRecursionInAlias { alias: SmolStr, token: SmolStr },

    #[error("Unexpected token `{token}` in alias `{alias}`")]
    UnexpectedTokenInAlias { alias: SmolStr, token: SmolStr },

    #[error("Unknown alias `{alias}`")]
    UnknownAlias { alias: SmolStr },

    #[error("Missing token(s) in binding `{binding}`")]
    MissingTokenInBinding { binding: SmolStr },

    #[error("Unexpected token `{token}` in binding `{binding}`")]
    UnexpectedTokenInBinding { binding: SmolStr, token: SmolStr },

    #[error("Conflicting binding `{binding}`")]
    ConflictingBinding { binding: SmolStr },

    #[error("Duplicated count in binding {binding}")]
    DuplicatedCountInBinding { binding: SmolStr },

    #[error("Unused count in binding {binding}")]
    UnusedCountInBinding { binding: SmolStr },

    #[error("Unexpected unstick in action `{action}`")]
    UnexpectedUnstickInAction { action: SmolStr },

    #[error("Unknown action `{action}`")]
    UnknownAction { action: SmolStr },

    #[error("Unexpected token `{token}` in action `{action}`")]
    UnexpectedTokenInAction { action: SmolStr, token: SmolStr },

    #[error("Invalid argument `{argument}` in action `{action}`")]
    InvalidArgumentInAction { action: SmolStr, argument: SmolStr },

    #[error("Unknown argument `{argument}` in action `{action}`")]
    UnknownArgumentInAction { action: SmolStr, argument: SmolStr },

    #[error("Unknown count in action `{action}`")]
    UnknownCountInAction { action: SmolStr },
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           ModdedKey                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// [`Mods`] and [`Key`].
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ModdedKey<T = SmolStr> {
    mods: Mods,
    key: Key<T>,
}

impl<T> ModdedKey<T> {
    pub fn new(mods: Mods, key: Key<T>) -> Self {
        Self { mods, key }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ModdedKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.mods, f)?;
        std::fmt::Debug::fmt(&self.key, f)?;

        Ok(())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Count                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

use self::Count::*;

/// [`Count::Optional`] or [`Count::Required`].
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Count<T> {
    Optional(T),
    Required(T),
}

impl<T> Count<T> {
    fn unwrap(self) -> T {
        match self {
            Optional(count) => count,
            Required(count) => count,
        }
    }
}

impl<T: AsRef<str>> Count<T> {
    fn to_smol(&self) -> Count<SmolStr> {
        match self {
            Optional(str) => Optional(SmolStr::new(str)),
            Required(str) => Required(SmolStr::new(str)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Count<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Optional(count) => write!(f, "?{count:?}"),
            Required(count) => write!(f, "?!{count:?}"),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           CountOrKey                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

use CountOrKey::*;

/// [`CountOrKey::Count`] or [`CountOrKey::Key`].
#[derive(Clone, Eq, PartialEq, Hash)]
enum CountOrKey<T = ()> {
    Count(Count<T>),
    Key(ModdedKey),
}

impl<T: std::fmt::Debug> std::fmt::Debug for CountOrKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Count(count) => std::fmt::Debug::fmt(count, f),
            Key(key) => std::fmt::Debug::fmt(key, f),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Action                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Generates [`Action`] and related stuff.
macro_rules! actions {
    ($(
        $(#[$action_meta:meta])*
        $action:ident($(
            $(#[$arg_meta:meta])*
            $arg:ident: ($($ty:tt)*) $(= $default:literal)? $(,)?
        ),*)
    ),* $(,)?) => {
        /// An [`Action`] parsed with [`ParsedArgument`]s.
        #[derive(Clone, Eq, PartialEq, Debug)]
        #[allow(non_camel_case_types)]
        enum ParsedAction { $(
            $(#[$action_meta])*
            $action { $(
                $(#[$arg_meta])*
                $arg: ParsedArgument,
            )* },
        )* }

        impl ParsedAction {
            /// Returns the default `action` [`ParsedAction`].
            fn default(action: &str) -> Option<Self> {
                match action {
                    $(
                        $(#[$action_meta])*
                        stringify!($action) => Some(ParsedAction::$action { $(
                            $arg: Default::default(),
                        )* }),
                    )*
                    _ => None,
                }
            }

            /// Sets argument's `name` with `value` or errors on type mismatch.
            fn arg(&mut self, name: &str, value: ParsedArgument) -> KeybindingsResult<()> {
                match self { $(
                    $(#[$action_meta])*
                    Self::$action { $($arg,)* } =>
                        match name {
                            $(
                                stringify!($arg) =>
                                    if actions!(impl satisfies for $($ty)*)(value) {
                                        *$arg = value;
                                    } else {
                                        return Err(InvalidArgumentInAction {
                                            action: stringify!($action).to_smolstr(),
                                            argument: name.to_smolstr(),
                                        });
                                    }
                            ),*
                            _ => return Err(UnknownArgumentInAction {
                                action: stringify!($action).to_smolstr(),
                                argument: name.to_smolstr(),
                            }),
                        }
                )* }

                Ok(())
            }

            /// Resolves arguments with `counts`.
            fn action(self, counts: &HashMap<NodeId, Number>) -> Action {
                match self { $(
                    $(#[$action_meta])*
                    Self::$action { $($arg,)* } =>
                        Action::$action { $(
                            $arg: actions!(impl unparse_arg for $($ty)* $(= $default)?)($arg, counts, $($default)?),
                        )* },
                )* }
            }
        }

        /// `Virus`'s actions.
        #[derive(Eq, PartialEq, Debug)]
        #[allow(non_camel_case_types)]
        pub enum Action { $(
            $(#[$action_meta])*
            $action { $(
                $(#[$arg_meta])*
                $arg: actions!(impl arg_call_type for $($ty)* $(= $default)?),
            )* },
        )* }

        pub trait ActionHandler {
            /// Handles `action`.
            fn handle(&mut self, action: Action) {
                match action { $(
                    $(#[$action_meta])*
                    Action::$action { $($arg,)* } => self.$action($($arg,)*),
                )* }
            }

            $(
                $(#[$action_meta])*
                fn $action(
                    &mut self,
                    $(
                        $arg: actions!(impl arg_call_type for $($ty)* $(= $default)?),
                    )*
                );
            )*
        }
    };

    (impl arg_call_type for Number) => { Number };
    (impl arg_call_type for bool) => { bool };
    (impl arg_call_type for Option<Number>) => { Option<Number> };
    (impl arg_call_type for Option<bool>) => { Option<bool> };
    (impl arg_call_type for Option<Number> = $_default:literal) => { Number };
    (impl arg_call_type for Option<bool> = $_default:literal) => { bool };

    (impl satisfies for Number) => { ParsedArgument::satisfies_number };
    (impl satisfies for bool) => { ParsedArgument::satisfies_bool };
    (impl satisfies for Option<Number>) => { ParsedArgument::satisfies_option_number };
    (impl satisfies for Option<bool>) => { ParsedArgument::satisfies_option_bool };

    (impl unparse_arg for Number) => { ParsedArgument::unparse_number };
    (impl unparse_arg for bool) => { ParsedArgument::unparse_bool };
    (impl unparse_arg for Option<Number>) => { ParsedArgument::unparse_option_number };
    (impl unparse_arg for Option<bool>) => { ParsedArgument::unparse_option_bool };
    (impl unparse_arg for Option<Number> = $default:literal) => { ParsedArgument::unparse_option_number_default };
    (impl unparse_arg for Option<bool> = $default:literal) => { ParsedArgument::unparse_option_bool_default };
}

actions!(
    //
    // MOVE
    //

    // MOVE up

    move_top(
        blank: (Option<bool>) = false,
    ),

    move_up_page(
        pages: (Option<Number>) = 1,
        half: (Option<bool>) = false,
        wrap: (Option<bool>) = false,
        // TODO blank?
    ),

    move_up_line(
        lines: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    // MOVE down

    move_bottom(
        blank: (Option<bool>) = false,
    ),

    move_down_page(
        pages: (Option<Number>) = 1,
        half: (Option<bool>) = false,
        wrap: (Option<bool>) = false,
    ),

    move_down_line(
        lines: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    // MOVE left

    move_start(
        blank: (Option<bool>) = false,
    ),

    move_left_boundary(
        boundaries: (Option<Number>) = 1,
        punctuation_start: (Option<bool>) = false,
        punctuation_end: (Option<bool>) = false,
        short_word_start: (Option<bool>) = false,
        short_word_end: (Option<bool>) = false,
        long_word_start: (Option<bool>) = false,
        long_word_end: (Option<bool>) = false,
        wrap: (Option<bool>) = false,
    ),

    move_left_char(
        chars: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    move_left_smart(
        repeat: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    // MOVE right

    move_end(
        blank: (Option<bool>) = false,
    ),

    move_right_boundary(
        boundaries: (Option<Number>) = 1,
        punctuation_start: (Option<bool>) = false,
        punctuation_end: (Option<bool>) = false,
        short_word_start: (Option<bool>) = false,
        short_word_end: (Option<bool>) = false,
        long_word_start: (Option<bool>) = false,
        long_word_end: (Option<bool>) = false,
        wrap: (Option<bool>) = false,
    ),

    move_right_char(
        chars: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    move_right_smart(
        repeat: (Option<Number>) = 1,
        wrap: (Option<bool>) = false,
    ),

    //
    // SCROLL
    //

    // SCROLL up

    scroll_top(
        blank: (Option<bool>) = false,
    ),

    scroll_up_page(
        pages: (Option<Number>) = 1,
        half: (Option<bool>) = false,
        blank: (Option<bool>) = false,
    ),

    scroll_up_line(
        lines: (Option<Number>) = 1,
        blank: (Option<bool>) = false,
    ),

    // SCROLL down

    scroll_bottom(
        blank: (Option<bool>) = false,
    ),

    scroll_down_page(
        pages: (Option<Number>) = 1,
        half: (Option<bool>) = false,
        blank: (Option<bool>) = false,
    ),

    scroll_down_line(
        lines: (Option<Number>) = 1,
        blank: (Option<bool>) = false,
    ),

    // SCROLL align

    scroll_align_top(
        margin: (Option<Number>) = 0,
    ),

    scroll_align_center(),

    scroll_align_bottom(
        margin: (Option<Number>) = 0,
    ),

    //
    // Selection
    //

    select(
        lines: (Option<bool>) = false,
    ),

    select_smart(),

    flip_selection(),

    unselect(
        anchor: (Option<bool>) = false,
    ),

    //
    // Modes
    //

    normal(),

    insert(),

    files(),

    //
    //
    //

    cut(),

    copy(),

    paste(),

    undo(),

    redo(),

    open(),

    save(),

    close(),

    /// An action for tests.
    #[cfg(test)]
    test(
        /// Some value to test against.
        value: (Option<Number>) = 0,
    ),
);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         ParsedArgument                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// An [`Action`] argument parsed from keybindings config for [`ParsedAction`].
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
enum ParsedArgument {
    /// `?count` or `?!count`, with [`NodeId`] of the corresponding [`CountOrKey::Count`] node.
    Count(Count<NodeId>),
    /// `value=12`.
    Number(Number),
    /// `+wrap` or `-wrap`.
    Bool(bool),
    /// For optional arguments.
    #[default]
    None,
}

impl ParsedArgument {
    fn satisfies_number(self) -> bool {
        matches!(self, Self::Count(Required(_)) | Self::Number(_))
    }

    fn satisfies_option_number(self) -> bool {
        matches!(self, Self::Count(_) | Self::Number(_) | Self::None)
    }

    fn satisfies_bool(self) -> bool {
        matches!(self, Self::Bool(_))
    }

    fn satisfies_option_bool(self) -> bool {
        matches!(self, Self::Bool(_) | Self::None)
    }

    fn unparse_number(self, counts: &HashMap<NodeId, Number>) -> Number {
        match self {
            ParsedArgument::Count(id) => counts.get(&id.unwrap()).copied().unwrap(),
            ParsedArgument::Number(number) => number,
            ParsedArgument::Bool(_) => unreachable!(),
            ParsedArgument::None => unreachable!(),
        }
    }

    fn unparse_bool(self, counts: &HashMap<NodeId, Number>) -> bool {
        match self {
            ParsedArgument::Count(_) => unreachable!(),
            ParsedArgument::Number(_) => unreachable!(),
            ParsedArgument::Bool(bool) => bool,
            ParsedArgument::None => unreachable!(),
        }
    }

    fn unparse_option_number(self, counts: &HashMap<NodeId, Number>) -> Option<Number> {
        match self {
            ParsedArgument::Count(id) => counts.get(&id.unwrap()).copied(),
            ParsedArgument::Number(number) => Some(number),
            ParsedArgument::Bool(_) => unreachable!(),
            ParsedArgument::None => None,
        }
    }

    fn unparse_option_bool(self, counts: &HashMap<NodeId, Number>) -> Option<bool> {
        match self {
            ParsedArgument::Count(_) => unreachable!(),
            ParsedArgument::Number(_) => unreachable!(),
            ParsedArgument::Bool(bool) => Some(bool),
            ParsedArgument::None => None,
        }
    }

    fn unparse_option_number_default(
        self,
        counts: &HashMap<NodeId, Number>,
        default: Number,
    ) -> Number {
        match self {
            ParsedArgument::Count(id) => counts.get(&id.unwrap()).copied().unwrap_or(default),
            ParsedArgument::Number(number) => number,
            ParsedArgument::Bool(_) => unreachable!(),
            ParsedArgument::None => default,
        }
    }

    fn unparse_option_bool_default(self, counts: &HashMap<NodeId, Number>, default: bool) -> bool {
        match self {
            ParsedArgument::Count(_) => unreachable!(),
            ParsedArgument::Number(_) => unreachable!(),
            ParsedArgument::Bool(bool) => bool,
            ParsedArgument::None => default,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Keybindings                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type Aliases = HashMap<SmolStr, Vec<ModdedKey>>;

#[derive(Eq, PartialEq, Debug)]
pub struct Keybindings {
    mode: Mode,
    aliases: Aliases,
    normal: Nodes,
    insert: Nodes,
    files: Nodes,
}

impl Keybindings {
    /// Deserializes [`Keybindings`] from `yaml`.
    pub fn from_yaml(yaml: &str) -> KeybindingsResult<Self> {
        Self::try_from(serde_yaml::from_str::<deserialize::Keybindings>(yaml)?)
    }

    /// Resets to `mode`.
    pub fn mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.normal.reset();
        self.insert.reset();
        self.files.reset();
    }

    /// Handles an `event`, returning:
    /// - `Err(())` when no bindings accepts `event`
    /// - `Ok(None)` when a binding accepts `event` without triggering an action
    /// - `Ok(Some(action))` when a binding accepts `event` to trigger `action`
    pub fn handle(&mut self, event: &KeyEvent) -> Result<Option<Action>, ()> {
        match self.mode {
            Mode::Normal { .. } => self.normal.handle(event),
            Mode::Insert { .. } => self.insert.handle(event),
            Mode::Files => self.files.handle(event),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::from_yaml(&deserialize::DEFAULT).unwrap()
    }
}

impl TryFrom<deserialize::Keybindings> for Keybindings {
    type Error = KeybindingsError;

    fn try_from(keybindings: deserialize::Keybindings) -> Result<Self, Self::Error> {
        let aliases = keybindings
            .aliases
            .into_iter()
            .map(|(alias, keys)| {
                Ok((
                    SmolStr::new(&alias),
                    parse::alias(&alias, &keys).collect::<Result<_, _>>()?,
                ))
            })
            .collect::<Result<_, Self::Error>>()?;
        let normal = Nodes::try_from((&aliases, keybindings.normal))?;
        let insert = Nodes::try_from((&aliases, keybindings.insert))?;
        let files = Nodes::try_from((&aliases, keybindings.files))?;

        Ok(Self {
            mode: Default::default(),
            aliases,
            normal,
            insert,
            files,
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Nodes                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Keybindings finite state machine.
#[derive(Clone, Eq, PartialEq, Default, Debug)]
struct Nodes {
    nodes: HashMap<NodeId, Node>,
    current: NodeId,
    counts: HashMap<NodeId, Number>,
}

impl Nodes {
    /// Handles `event`.
    fn handle(&mut self, event: &KeyEvent) -> Result<Option<Action>, ()> {
        // NOTE:
        // I wonder if it's possible to have different keys when no shift nor alt are pressed
        debug_assert!((!event.shift() && !event.alt())
            .then(|| event.modded() == event.unmodded())
            .unwrap_or(true));

        let get_children = |id| match self.nodes.get(&id).unwrap() {
            Node::Node { children, .. } => children,
            _ => unreachable!(),
        };
        let children = get_children(self.current);
        let children_after_optional = children
            .get(&Count(Optional(())))
            .map(|id| get_children(*id));

        // Parse event as number for count nodes
        if let Some(current) = None
            .or(children.get(&Count(Optional(()))))
            .or(children.get(&Count(Required(()))))
        {
            if let Key::Str(str) = event.modded() {
                if let Ok(number) = str.parse::<Number>() {
                    let count = self.counts.entry(*current).or_default();
                    *count *= (10 as Number).pow(str.len() as u32);
                    *count += number;

                    self.current = *current;
                    return Ok(None);
                }
            }
        }

        // Transition state machine if a binding is matched
        let current = if event.modded() == event.unmodded() {
            let modded = Key(ModdedKey::new(
                Mods::new(event.control(), event.shift(), event.alt(), event.command()),
                event.modded().clone(),
            ));

            None.or_else(|| children.get(&modded))
                .or_else(|| children_after_optional.and_then(|children| children.get(&modded)))
        } else {
            // NOTE: What to do here?
            // With `control N: action`,
            // receiving `{ mods: CONTROL | SHIFT, modded: N, unmodded: n }` must match!
            // Here we suppose only shift or alt can modify the key,
            // so we try modded without these then unmodded with these.
            // I guess this leads to unexpected edge-cases but at this point I don't even care.

            let modded = Key(ModdedKey::new(
                Mods::new(event.control(), false, false, event.command()),
                event.modded().clone(),
            ));
            let unmodded = Key(ModdedKey::new(
                Mods::new(event.control(), event.shift(), event.alt(), event.command()),
                event.unmodded().clone(),
            ));

            None.or_else(|| children.get(&modded))
                .or_else(|| children_after_optional.and_then(|children| children.get(&modded)))
                .or_else(|| children.get(&unmodded))
                .or_else(|| children_after_optional.and_then(|children| children.get(&unmodded)))
        };

        self.current = if let Some(&current) = current {
            current
        } else {
            if *event.modded() == Key::Escape
                && !event.control()
                && !event.shift()
                && !event.alt()
                && !event.command()
            {
                self.escape();
                return Ok(None);
            } else {
                return Err(());
            }
        };

        // Return action
        match self.nodes.get(&self.current).unwrap() {
            Node::Node { .. } => Ok(None),
            Node::Leaf { action, .. } => {
                let action = action.clone().map(|action| action.action(&self.counts));

                self.escape();
                Ok(action)
            }
        }
    }

    /// Climbs the branch up until unstuck properly, removing counts on the way.
    fn escape(&mut self) {
        self.counts.remove(&self.current);

        let mut sticky;
        let (mut parent, mut unstick) = match self.nodes.get(&self.current).unwrap() {
            Node::Node { parent, .. } => (*parent, false),
            Node::Leaf {
                parent, unstick, ..
            } => (*parent, *unstick),
        };

        while let Some(id) = parent {
            self.current = id;
            self.counts.remove(&self.current);

            (parent, sticky) = match self.nodes.get(&self.current).unwrap() {
                Node::Node { parent, sticky, .. } => (*parent, *sticky),
                Node::Leaf { .. } => unreachable!(),
            };

            if sticky {
                if unstick {
                    unstick = false;
                } else {
                    break;
                }
            }
        }
    }

    /// Resets to the initial state.
    fn reset(&mut self) {
        self.current = 0;
        self.counts.clear();
    }
}

impl TryFrom<(&Aliases, deserialize::Nodes)> for Nodes {
    type Error = KeybindingsError;

    /// Constructs the nodes tree recursively.
    fn try_from((aliases, nodes): (&Aliases, deserialize::Nodes)) -> Result<Self, Self::Error> {
        struct Context<'a> {
            aliases: &'a Aliases,
            nodes: HashMap<NodeId, Node>,
            id: NodeId,
            /// Keeps track of counts in the current branch,
            /// giving the corresponding node id
            /// and a flag to check if the count is actually used in the leaf action.
            counts: HashMap<Count<SmolStr>, (NodeId, /* used */ bool)>,
        }

        macro_rules! insert {
            ($self:ident, $id:ident, $node:expr) => {{
                let $id = {
                    let id = $self.id;
                    $self.id += 1;
                    id
                };
                let node = { $node };

                debug_assert!($self.nodes.get(&$id).is_none());
                $self.nodes.insert($id, node);

                $id
            }};
        }

        impl<'a> Context<'a> {
            fn new(aliases: &'a Aliases) -> Self {
                Self {
                    aliases,
                    nodes: Default::default(),
                    id: 0,
                    counts: Default::default(),
                }
            }

            fn get_children(node: &Node) -> &HashMap<CountOrKey, NodeId> {
                match node {
                    Node::Node { children, .. } => children,
                    Node::Leaf { .. } => unreachable!(),
                }
            }

            fn get_children_mut(node: &mut Node) -> &mut HashMap<CountOrKey, NodeId> {
                match node {
                    Node::Node { children, .. } => children,
                    Node::Leaf { .. } => unreachable!(),
                }
            }

            /// Constructs the nodes tree recursively.
            fn convert(mut self, nodes: deserialize::Nodes) -> KeybindingsResult<Nodes> {
                let current = insert!(
                    self,
                    id,
                    self.node(false, None, id, None, false, nodes.children)?
                );

                debug_assert_eq!(current, 0);

                Ok(Nodes {
                    nodes: self.nodes,
                    current,
                    counts: Default::default(),
                })
            }

            fn leaf(
                &mut self,
                has_sticky: bool,
                parent: Option<NodeId>,
                action_str: String,
            ) -> KeybindingsResult<Node> {
                let (unstick, action) = parse::action(&action_str, |count| {
                    self.counts
                        .get_mut(&count.to_smol())
                        .ok_or_else(|| UnknownCountInAction {
                            action: action_str.to_smolstr(),
                        })
                        .map(|(count, used)| {
                            // Mark that count as used
                            *used = true;
                            *count
                        })
                })?;

                if unstick && !has_sticky {
                    return Err(UnexpectedUnstickInAction {
                        action: action_str.to_smolstr(),
                    });
                }

                Ok(Node::Leaf {
                    parent,
                    unstick,
                    action,
                })
            }

            fn node(
                &mut self,
                has_sticky: bool,
                parent: Option<NodeId>,
                node_id: NodeId,
                text: Option<String>,
                sticky: bool,
                deserialized_children: HashMap<String, deserialize::Node>,
            ) -> KeybindingsResult<Node> {
                // The node to create
                let mut node = Node::Node {
                    parent,
                    text,
                    sticky,
                    children: Default::default(),
                };
                let has_sticky = has_sticky || sticky;

                for (binding, deserialized_child) in deserialized_children {
                    // Here we have the optional count and keys, i.e.:
                    //
                    // `?!count $up: action` -> (Some(Required("count")), [Str("i"), ArrowUp])
                    //
                    // Which must give:
                    //
                    //                          node
                    //                           |
                    //                       Required
                    //                       /      \
                    //                      "i"  ArrowUp
                    //                       \      /
                    //                        action
                    //
                    let (count, keys) = parse::binding(&binding, self.aliases)?;

                    // Create the count node
                    let parent_id = if let Some(count) = count {
                        let id = *Self::get_children_mut(&mut node)
                            .entry(Count(match count {
                                Optional(count) => Optional(()),
                                Required(count) => Required(()),
                            }))
                            .or_insert_with(|| {
                                insert!(
                                    self,
                                    id,
                                    Node::Node {
                                        parent: Some(node_id),
                                        text: Default::default(),
                                        children: [(Count(Optional(())), id)].into_iter().collect(),
                                        sticky: false
                                    }
                                )
                            });

                        let count = count.to_smol();

                        // It's an error to have another count with the same name in that branch
                        if false
                            || self.counts.get(&Optional(count.clone().unwrap())).is_some()
                            || self.counts.get(&Required(count.clone().unwrap())).is_some()
                        {
                            return Err(DuplicatedCountInBinding {
                                binding: binding.to_smolstr(),
                            });
                        }

                        // Keep track of the count for that branch
                        self.counts.insert(count, (id, false));

                        Some(id)
                    } else {
                        None
                    };

                    // Create the child node
                    let child_id = {
                        let parent = parent_id.or(Some(node_id));

                        insert!(
                            self,
                            id,
                            match deserialized_child {
                                deserialize::Node::Node {
                                    text,
                                    sticky,
                                    children,
                                } => self.node(has_sticky, parent, id, text, sticky, children),
                                deserialize::Node::Leaf(action) => {
                                    self.leaf(has_sticky, parent, action)
                                }
                            }?
                        )
                    };

                    // Attach the child to all keys in the binding
                    for key in keys.map(Key) {
                        // Children of the currently created node or its above count child node
                        let children = match parent_id {
                            Some(id) => Self::get_children_mut(self.nodes.get_mut(&id).unwrap()),
                            None => Self::get_children_mut(&mut node),
                        };

                        match children.get(&key) {
                            Some(_) => {
                                return Err(ConflictingBinding {
                                    binding: binding.to_smolstr(),
                                });
                            }
                            None => _ = children.insert(key, child_id),
                        }
                    }

                    // Check count usage and clean
                    if let Some(count) = count.map(|count| count.to_smol()) {
                        if !self.counts.get(&count).unwrap().1 {
                            return Err(UnusedCountInBinding {
                                binding: binding.to_smolstr(),
                            });
                        }

                        self.counts.remove(&count);
                    }

                    // Check for conflicts
                    if self.conflicting_bindings(&node) {
                        return Err(ConflictingBinding {
                            binding: binding.to_smolstr(),
                        });
                    }
                }

                Ok(node)
            }

            fn conflicting_bindings(&self, node: &Node) -> bool {
                let children = Self::get_children(&node);
                let optional = children.get(&Count(Optional(())));
                let required = children.get(&Count(Required(())));
                let has_parseable = |nodes: &HashMap<CountOrKey, NodeId>| {
                    nodes
                        .keys()
                        .filter(|key| match key {
                            Key(ModdedKey {
                                key: Key::Str(str), ..
                            }) => str.parse::<Number>().is_ok(),
                            _ => false,
                        })
                        .next()
                        .is_some()
                };

                // Both optional and required count:
                // how could we know which count to go to when we handle a number event?
                if optional.is_some() && required.is_some() {
                    return true;
                }

                // Having a count and a number at the same level: same issue.
                if (optional.is_some() || required.is_some()) && has_parseable(children) {
                    return true;
                }

                // Having a count before a number key:
                // how do we know when count stops/the number starts?
                if let Some(count_children) = optional
                    .or(required)
                    .map(|id| Self::get_children(self.nodes.get(id).unwrap()))
                {
                    if has_parseable(count_children) {
                        return true;
                    }
                }

                // A key with and without an optional count at the same level:
                // when we only receive the key, which binding to go to?
                if let Some(optional_children) =
                    optional.map(|id| Self::get_children(self.nodes.get(id).unwrap()))
                {
                    for key in optional_children.keys() {
                        if matches!(key, Key(key) if children.get(&Key(key.clone())).is_some()) {
                            return true;
                        }
                    }
                }

                false
            }
        }

        Context::new(aliases).convert(nodes)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Node                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A node in [`Nodes`].
#[derive(Clone, Eq, PartialEq, Debug)]
enum Node {
    Node {
        parent: Option<NodeId>,
        text: Option<String>,
        sticky: bool,
        children: HashMap<CountOrKey, NodeId>,
    },
    Leaf {
        parent: Option<NodeId>,
        unstick: bool,
        action: Option<ParsedAction>,
    },
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Deserialize                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

mod deserialize {
    use super::*;

    pub const DEFAULT: &'static str = include_str!("keybindings.yml");

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "UPPERCASE")]
    #[serde(deny_unknown_fields)]
    pub struct Keybindings {
        #[serde(default)]
        pub aliases: HashMap<String, String>,
        #[serde(default)]
        pub normal: Nodes,
        #[serde(default)]
        pub insert: Nodes,
        #[serde(default)]
        pub files: Nodes,
    }

    #[derive(Deserialize, Clone, Default, Debug)]
    pub struct Nodes {
        #[serde(flatten)]
        pub children: HashMap<String, Node>,
    }

    #[derive(Deserialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum Node {
        Node {
            #[serde(rename = "_text")]
            text: Option<String>,
            #[serde(rename = "_sticky")]
            #[serde(default)]
            sticky: bool,
            #[serde(flatten)]
            children: HashMap<String, Node>,
        },
        Leaf(String),
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Parse                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

mod parse {
    use super::*;

    /// Parseable tokens from keybindings config.
    #[derive(Copy, Clone, Debug)]
    enum Token<'a> {
        Str(&'a str),
        Dollar(&'a str),
        Underscore(&'a str),
        QuestionExclam(&'a str),
        Question(&'a str),
        Plus(&'a str),
        Minus(&'a str),
        EqualNumber(&'a str, Number),
        EqualQuestionExclam(&'a str, &'a str),
        EqualQuestion(&'a str, &'a str),
    }

    impl<'a> std::fmt::Display for Token<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Token::Str(str) => write!(f, "{str}"),
                Token::Dollar(str) => write!(f, "${str}"),
                Token::Underscore(str) => write!(f, "_{str}"),
                Token::QuestionExclam(str) => write!(f, "?!{str}"),
                Token::Question(str) => write!(f, "?{str}"),
                Token::Plus(str) => write!(f, "+{str}"),
                Token::Minus(str) => write!(f, "-{str}"),
                Token::EqualNumber(left, right) => write!(f, "{left}={right}"),
                Token::EqualQuestionExclam(left, right) => write!(f, "{left}=?!{right}"),
                Token::EqualQuestion(left, right) => write!(f, "{left}=?{right}"),
            }
        }
    }

    /// [`Token`] iterator.
    struct Tokens<'a> {
        words: Filter<Split<'a, char>, fn(&&str) -> bool>,
    }

    impl<'a> Tokens<'a> {
        fn new(str: &'a str) -> Self {
            Self {
                words: str.split(' ').filter(|part| !part.is_empty()),
            }
        }
    }

    impl<'a> Iterator for Tokens<'a> {
        type Item = KeybindingsResult<Token<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            // TODO validate better, or don't
            fn validate_ident(identifier: &str) -> KeybindingsResult<&str> {
                if identifier.is_empty() {
                    return Err(InvalidIdentifier {
                        identifier: identifier.to_smolstr(),
                    });
                }

                // for byte in identifier.bytes() {
                //     if !matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') {
                //         return Err(InvalidIdentifier {
                //             identifier: identifier.to_smolstr(),
                //         });
                //     }
                // }

                Ok(identifier)
            }

            fn strip<'a>(
                word: &'a str,
                prefix: &str,
                variant: impl Fn(&'a str) -> Token<'a>,
            ) -> Option<KeybindingsResult<Token<'a>>> {
                word.strip_prefix(prefix)
                    .map(|word| validate_ident(word).map(variant))
            }

            let word = self.words.next()?;

            if let Some(token) = strip(word, "$", Token::Dollar) {
                return Some(token);
            }

            if let Some(token) = strip(word, "_", Token::Underscore) {
                return Some(token);
            }

            if let Some(token) = strip(word, "?!", Token::QuestionExclam) {
                return Some(token);
            }

            if let Some(token) = strip(word, "?", Token::Question) {
                return Some(token);
            }

            if let Some(token) = strip(word, "+", Token::Plus) {
                return Some(token);
            }

            if let Some(token) = strip(word, "-", Token::Minus) {
                return Some(token);
            }

            if let Some((word, value)) = word.split_once('=') {
                let word = match validate_ident(word) {
                    Ok(word) => word,
                    Err(err) => return Some(Err(err)),
                };

                if let Ok(value) = value.parse::<Number>() {
                    return Some(Ok(Token::EqualNumber(word, value)));
                }

                // NOTE: we could disallow `value=?!value` to enforce compact form
                if let Some(token) =
                    strip(value, "?!", |value| Token::EqualQuestionExclam(word, value))
                {
                    return Some(token);
                }

                // NOTE: we could disallow `value=?value` to enforce compact form
                if let Some(token) = strip(value, "?", |value| Token::EqualQuestion(word, value)) {
                    return Some(token);
                }

                return Some(Err(InvalidIdentifier {
                    identifier: value.to_smolstr(),
                }));
            }

            Some(validate_ident(word).map(Token::Str))
        }
    }

    /// Parses [`Mods`] from `tokens`.
    fn mods(tokens: &mut Peekable<Tokens>) -> Mods {
        let [mut control, mut shift, mut alt, mut command] = [false; 4];

        loop {
            match tokens.peek() {
                Some(Ok(Token::Str(str))) if *str == Mods::CONTROL_STR => control = true,
                Some(Ok(Token::Str(str))) if *str == Mods::SHIFT_STR => shift = true,
                Some(Ok(Token::Str(str))) if *str == Mods::ALT_STR => alt = true,
                Some(Ok(Token::Str(str))) if *str == Mods::COMMAND_STR => command = true,
                _ => break,
            };

            tokens.next();
        }

        Mods::new(control, shift, alt, command)
    }

    /// Parses aliases keys in the form of `control a, arrow_up` from `str`.
    pub fn alias<'a>(
        alias: &'a str,
        str: &'a str,
    ) -> impl Iterator<Item = KeybindingsResult<ModdedKey>> + 'a {
        fn parse(alias: &str, str: &str) -> KeybindingsResult<ModdedKey> {
            let mut tokens = Tokens::new(str).peekable();
            let mods = self::mods(&mut tokens);
            let key = match tokens.next().transpose()? {
                Some(token @ Token::Dollar(_)) => {
                    return Err(UnimplementedRecursionInAlias {
                        alias: alias.to_smolstr(),
                        token: token.to_smolstr(),
                    });
                }
                // NOTE: any other tokens will be parsed as-is, e.g. `Key::Str("_weird")`
                Some(token) => Key::parse(&token.to_smolstr()).to_smol(),
                None => {
                    return Err(MissingTokenInAlias {
                        alias: alias.to_smolstr(),
                    });
                }
            };

            if let Some(token) = tokens.next() {
                return Err(UnexpectedTokenInAlias {
                    alias: alias.to_smolstr(),
                    token: token?.to_smolstr(),
                });
            }

            Ok(ModdedKey::new(mods, key))
        }

        str.split(',').map(|str| parse(alias, str))
    }

    /// Parses `?count $up` from `binding`, resolving aliases in `aliases`.
    pub fn binding<'a>(
        binding: &'a str,
        aliases: &'a Aliases,
    ) -> KeybindingsResult<(Option<Count<&'a str>>, impl Iterator<Item = ModdedKey> + 'a)> {
        let mut tokens = Tokens::new(binding).peekable();
        let count = tokens
            .peek()
            .map(|token| match token {
                Ok(Token::Question(str)) => Some(Optional(*str)),
                Ok(Token::QuestionExclam(str)) => Some(Required(*str)),
                _ => None,
            })
            .flatten()
            .inspect(|_| {
                tokens.next();
            });
        let mods = self::mods(&mut tokens);

        enum OneOrMany<'a> {
            One(Option<ModdedKey>),
            Many(Iter<'a, ModdedKey>),
        }

        let mut keys = match tokens.next().transpose()? {
            // Resolve aliases
            Some(Token::Dollar(alias)) => match aliases.get(alias) {
                Some(aliases) => OneOrMany::Many(aliases.iter()),
                None => {
                    return Err(UnknownAlias {
                        alias: alias.to_smolstr(),
                    });
                }
            },
            // NOTE: any other tokens will be parsed as-is, e.g. `Key::Str("_weird")`
            Some(token) => OneOrMany::One(Some(ModdedKey::new(
                mods,
                Key::parse(&token.to_smolstr()).to_smol(),
            ))),
            None => {
                return Err(MissingTokenInBinding {
                    binding: binding.to_smolstr(),
                });
            }
        };

        if let Some(token) = tokens.next() {
            return Err(UnexpectedTokenInBinding {
                binding: binding.to_smolstr(),
                token: token?.to_smolstr(),
            });
        }

        Ok((
            count,
            std::iter::from_fn(move || match &mut keys {
                OneOrMany::One(option) => option.take(),
                OneOrMany::Many(iter) => iter
                    .next()
                    .cloned()
                    .map(|mut key| ModdedKey::new(key.mods | mods, key.key)),
            }),
        ))
    }

    /// Parses `_unstick test value=?count` from `action`,
    /// resolving count node id with `count_by_name`.
    pub fn action(
        action_str: &str,
        mut count_by_name: impl FnMut(Count<&str>) -> KeybindingsResult<NodeId>,
    ) -> KeybindingsResult<(bool, Option<ParsedAction>)> {
        let mut tokens = Tokens::new(action_str).peekable();
        let unstick = match tokens.peek() {
            Some(Ok(Token::Underscore(str))) if *str == UNSTICK => {
                tokens.next();
                true
            }
            _ => false,
        };
        let mut action = match tokens.next().transpose()? {
            Some(token) => {
                let action = token.to_smolstr();
                ParsedAction::default(&action).ok_or_else(|| UnknownAction { action })?
            }
            None => return Ok((unstick, None)),
        };

        for token in tokens {
            let (name, value) = match token? {
                token @ Token::Str(_) | token @ Token::Dollar(_) | token @ Token::Underscore(_) => {
                    return Err(UnexpectedTokenInAction {
                        action: action_str.to_smolstr(),
                        token: token.to_smolstr(),
                    });
                }
                Token::QuestionExclam(name) => (
                    name,
                    ParsedArgument::Count(Required(count_by_name(Required(name))?)),
                ),
                Token::Question(name) => (
                    name,
                    ParsedArgument::Count(Optional(count_by_name(Optional(name))?)),
                ),
                Token::Plus(name) => (name, ParsedArgument::Bool(true)),
                Token::Minus(name) => (name, ParsedArgument::Bool(false)),
                Token::EqualNumber(name, value) => (name, ParsedArgument::Number(value)),
                Token::EqualQuestionExclam(name, value) => (
                    name,
                    ParsedArgument::Count(Required(count_by_name(Required(value))?)),
                ),
                Token::EqualQuestion(name, value) => (
                    name,
                    ParsedArgument::Count(Optional(count_by_name(Optional(value))?)),
                ),
            };

            action.arg(name, value)?;
        }

        Ok((unstick, Some(action)))
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Tests                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Mods = Mods::NONE;
    const CONTROL: Mods = Mods::CONTROL;
    const SHIFT: Mods = Mods::SHIFT;
    const ALT: Mods = Mods::ALT;
    const COMMAND: Mods = Mods::COMMAND;

    use AssertCurrent::*;
    #[derive(Copy, Clone)]
    enum AssertCurrent {
        Root,
        Is(&'static str),
        At(&'static str),
    }

    fn assert<T: Copy + Into<Option<AssertCurrent>>>(
        nodes: &Nodes,
        prepare: &[(Mods, &str, &str, Result<Option<Number>, ()>, T)],
        events: &[(Mods, &str, &str, Result<Option<Number>, ()>, T)],
    ) {
        let mut nodes = nodes.clone();
        let mut currents = HashMap::new();

        for (mods, modded, unmodded, result, assert_current) in prepare {
            assert_eq!(
                nodes.handle(&KeyEvent::new(
                    *mods,
                    Key::parse(modded).to_smol(),
                    Key::parse(unmodded).to_smol(),
                )),
                result.map(|value| value.map(|value| Action::test { value })),
            );

            match assert_current.clone().into() {
                Some(Root) => assert_eq!(nodes.current, 0),
                Some(Is(is)) => assert_eq!(currents.insert(is, nodes.current), None),
                Some(At(at)) => assert_eq!(nodes.current, *currents.get(at).expect("at")),
                None => {}
            }
        }

        for (mods, modded, unmodded, result, assert_current) in events {
            let mut nodes = nodes.clone();

            assert_eq!(
                nodes.handle(&KeyEvent::new(
                    *mods,
                    Key::parse(modded).to_smol(),
                    Key::parse(unmodded).to_smol(),
                )),
                result.map(|value| value.map(|value| Action::test { value })),
            );

            match assert_current.clone().into() {
                Some(Root) => assert_eq!(nodes.current, 0),
                Some(Is(is)) => unreachable!(),
                Some(At(at)) => assert_eq!(nodes.current, *currents.get(at).expect("at")),
                None => {}
            }
        }
    }

    mod errors {
        use super::*;

        #[test]
        fn missing_token_in_alias_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        ALIASES:
                            my_alias:
                    "#,
                ),
                Err(MissingTokenInAlias { .. }),
            ));
        }

        #[test]
        fn missing_token_in_alias_2() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        ALIASES:
                            my_alias: control
                    "#,
                ),
                Err(MissingTokenInAlias { .. }),
            ));
        }

        #[test]
        fn unimplemented_recursion_in_alias_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        ALIASES:
                            up: arrow_up
                            big_up: shift $up
                    "#,
                ),
                Err(UnimplementedRecursionInAlias { .. }),
            ));
        }

        #[test]
        fn unexpected_token_in_alias_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        ALIASES:
                            my_alias: a b
                    "#,
                ),
                Err(UnexpectedTokenInAlias { .. }),
            ));
        }

        #[test]
        fn unknown_alias_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            $my_alias: test
                    "#,
                ),
                Err(UnknownAlias { .. }),
            ));
        }

        #[test]
        fn missing_token_in_binding_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            control: test
                    "#,
                ),
                Err(MissingTokenInBinding { .. }),
            ));
        }

        #[test]
        fn unexpected_token_in_binding_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            a b: test
                    "#,
                ),
                Err(UnexpectedTokenInBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        ALIASES:
                            up: arrow_up
                        NORMAL:
                            $up: test
                            arrow_up: test
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_2() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?value a: test ?value
                            a: test
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_3() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?value a: test ?value
                            1: test
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_4() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?!value a: test ?!value
                            1: test
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_5() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?value a: test ?value
                            ?!value b: test ?!value
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_6() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?value 1: test ?value
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn conflicting_binding_7() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?!value 1: test ?!value
                    "#,
                ),
                Err(ConflictingBinding { .. }),
            ));
        }

        #[test]
        fn duplicated_count_in_binding_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?a a:
                                ?a a: test value=?a
                    "#,
                ),
                Err(DuplicatedCountInBinding { .. }),
            ));
        }

        #[test]
        fn unused_count_in_binding_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?a a: test
                    "#,
                ),
                Err(UnusedCountInBinding { .. }),
            ));
        }

        #[test]
        fn unexpected_unstick_in_action_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            a: _unstick test
                    "#,
                ),
                Err(UnexpectedUnstickInAction { .. }),
            ));
        }

        #[test]
        fn unexpected_token_in_action_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            a: test foo $bar _baz
                    "#,
                ),
                Err(UnexpectedTokenInAction { .. }),
            ));
        }

        #[test]
        fn invalid_identifier_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            a: test value=false
                    "#,
                ),
                Err(InvalidIdentifier { .. }),
            ));
        }

        #[test]
        fn unknown_argument_in_action_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            a: test foo=0
                    "#,
                ),
                Err(UnknownArgumentInAction { .. }),
            ));
        }

        #[test]
        fn unknown_count_in_action_1() {
            assert!(matches!(
                Keybindings::from_yaml(
                    r#"
                        NORMAL:
                            ?lines a: test value=?rows
                    "#,
                ),
                Err(UnknownCountInAction { .. }),
            ));
        }
    }

    #[test]
    fn default() {
        Keybindings::default();
    }

    #[test]
    fn alias() {
        let mut nodes = Keybindings::from_yaml(
            r#"
                ALIASES:
                    up: u, arrow_up
                NORMAL:
                    $up: test value=1
            "#,
        )
        .unwrap()
        .normal;

        assert(
            &nodes,
            &[],
            &[
                (NONE, "u", "u", Ok(Some(1)), Root),               // Back to root
                (NONE, "arrow_up", "arrow_up", Ok(Some(1)), Root), // Back to root
                (NONE, "z", "z", Err(()), Root),                   // Unchanged
                (NONE, "escape", "escape", Ok(None), Root),        // Back to root
            ],
        );
    }

    #[test]
    fn sticky() {
        let mut nodes = Keybindings::from_yaml(
            r#"
                NORMAL:
                    a:
                        _sticky: true
                        b: test value=1
                        c: _unstick test value=2
                        d:
                            e: test value=3
                            f: _unstick test value=4
                            g:
                                _sticky: true
                                h: test value=5
                                i: _unstick test value=6
            "#,
        )
        .unwrap()
        .normal;

        assert(
            &nodes,
            &[],
            &[
                (NONE, "z", "z", Err(()), Root),            // Unchanged
                (NONE, "escape", "escape", Ok(None), Root), // Unstuck to root
            ],
        );

        assert(
            &nodes,
            &[(NONE, "a", "a", Ok(None), Is("a"))], // (sticky)
            &[
                (NONE, "b", "b", Ok(Some(1)), At("a")),     // Stuck in a
                (NONE, "c", "c", Ok(Some(2)), Root),        // Unstuck to root
                (NONE, "z", "z", Err(()), At("a")),         // Unchanged
                (NONE, "escape", "escape", Ok(None), Root), // Unstuck to root
            ],
        );

        assert(
            &nodes,
            &[
                (NONE, "a", "a", Ok(None), Is("a")), // (sticky)
                (NONE, "d", "d", Ok(None), Is("d")), // (not sticky)
            ],
            &[
                (NONE, "e", "e", Ok(Some(3)), At("a")),        // Stuck in a
                (NONE, "f", "f", Ok(Some(4)), Root),           // Unstuck to root
                (NONE, "z", "z", Err(()), At("d")),            // Unchanged
                (NONE, "escape", "escape", Ok(None), At("a")), // Unstuck to a
            ],
        );

        assert(
            &nodes,
            &[
                (NONE, "a", "a", Ok(None), Is("a")), // (sticky)
                (NONE, "d", "d", Ok(None), Is("d")), // (not sticky)
                (NONE, "g", "g", Ok(None), Is("g")), // (sticky)
            ],
            &[
                (NONE, "h", "h", Ok(Some(5)), At("g")),        // Stuck in g
                (NONE, "i", "i", Ok(Some(6)), At("a")),        // Unstuck to a
                (NONE, "z", "z", Err(()), At("g")),            // Unchanged
                (NONE, "escape", "escape", Ok(None), At("a")), // Unstuck to a
            ],
        );
    }

    #[test]
    fn count() {
        let mut nodes = Keybindings::from_yaml(
            r#"
                NORMAL:
                    ?value a: test ?value
            "#,
        )
        .unwrap()
        .normal;

        assert(
            &nodes,
            &[
                (NONE, "1", "1", Ok(None), Is("?value")),
                (NONE, "23", "23", Ok(None), At("?value")),
                (NONE, "4", "4", Ok(None), At("?value")),
            ],
            &[
                (NONE, "a", "a", Ok(Some(1234)), Root),
                (NONE, "z", "z", Err(()), At("?value")),
                (NONE, "escape", "escape", Ok(None), Root),
            ],
        );
    }

    #[test]
    fn sticky_count() {
        let mut nodes = Keybindings::from_yaml(
            r#"
                NORMAL:
                    ?value a:
                        _sticky: true
                        b:
                            c: test ?value
            "#,
        )
        .unwrap()
        .normal;

        assert(
            &nodes,
            &[
                (NONE, "1", "1", Ok(None), Is("?value")),
                (NONE, "3", "3", Ok(None), At("?value")),
                (NONE, "escape", "escape", Ok(None), Root),
                (NONE, "2", "2", Ok(None), At("?value")),
                (NONE, "a", "a", Ok(None), Is("a")),
                (NONE, "b", "b", Ok(None), Is("b")),
                (NONE, "escape", "escape", Ok(None), At("a")),
                (NONE, "b", "b", Ok(None), At("b")),
                (NONE, "c", "c", Ok(Some(2)), At("a")),
                (NONE, "b", "b", Ok(None), At("b")),
            ],
            &[
                (NONE, "c", "c", Ok(Some(2)), At("a")),
                (NONE, "z", "z", Err(()), At("b")),
                (NONE, "escape", "escape", Ok(None), At("a")),
            ],
        );
    }

    #[test]
    fn mods() {
        let mut nodes = Keybindings::from_yaml(
            r#"
                NORMAL:
                    shift a: test value=1
                    b: test value=2
                    B: test value=3
                    shift b: test value=4
                    shift c: test value=5
                    control c: test value=6
            "#,
        )
        .unwrap()
        .normal;

        assert(
            &nodes,
            &[],
            &[
                (SHIFT, "A", "a", Ok(Some(1)), Root),
                (NONE, "b", "b", Ok(Some(2)), Root),
                (SHIFT, "B", "b", Ok(Some(3)), Root),
                (NONE, "c", "c", Err(()), Root),
            ],
        );
    }
}
