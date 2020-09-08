//! Types for the intermediate representation of the macro input. This parsed
//! representation allows the functions in `gen.rs` to work more easily.

use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;


/// Input for the `write!` and `writeln!` macro.
#[derive(Debug)]
pub(crate) struct WriteInput {
    pub(crate) target: Expr,
    pub(crate) format_str: FormatStr,
    pub(crate) args: FormatArgs,
}

/// Our own `expr` type. We use this instead of `syn` to avoid `syn`
/// alltogether. We don't need to introspect the expression, we just need to
/// skip over them and the emit them again.
#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) span: Span,
    pub(crate) tokens: TokenStream,
}

/// A parsed format string.
#[derive(Debug)]
pub(crate) struct FormatStr {
    pub(crate) fragments: Vec<FormatStrFragment>,
}

impl FormatStr {
    /// Adds `\n` to the end of the formatting string.
    pub(crate) fn add_newline(&mut self) {
        match self.fragments.last_mut() {
            // If the last fragment is an `fmt` one, we can easily add the
            // newline to its last part (which is guaranteed to exist).
            Some(FormatStrFragment::Fmt { fmt_str_parts, .. }) => {
                fmt_str_parts.last_mut()
                    .expect("bug: fmt_str_parts empty")
                    .push('\n');
            }

            // Otherwise (style closing tag is last fragment), we have to add a
            // new `Fmt` fragment.
            _ => {
                self.fragments.push(FormatStrFragment::Fmt {
                    fmt_str_parts: vec!["\n".into()],
                    args: vec![],
                });
            }
        }
    }
}

/// One fragment of the format string.
#[derive(Debug)]
pub(crate) enum FormatStrFragment {
    /// A format string without style tags, but potentially with arguments.
    ///
    /// `fmt_str_parts` always has exactly one element more than `args`.
    Fmt {
        /// The format string as parts between the arguments.
        fmt_str_parts: Vec<String>,

        /// Information about argument that are referenced.
        args: Vec<ArgRef>,
    },

    /// A `{$...}` style start tag.
    StyleStart(Style),

    /// A `{/$}` style end tag.
    StyleEnd,
}

#[derive(Debug)]
pub(crate) struct ArgRef {
    pub(crate) kind: ArgRefKind,
    pub(crate) format_spec: String,
}

/// How a format argument is referred to.
#[derive(Debug)]
pub(crate) enum ArgRefKind {
    /// `{}`
    Next,
    /// `{2}`
    Position(u32),
    /// `{peter}`
    Name(String),
}

/// Parsed formatting arguments.
#[derive(Debug)]
pub(crate) struct FormatArgs {
    pub(crate) positional: Vec<Expr>,
    pub(crate) named: HashMap<String, Expr>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Color {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
    //Ansi256(u8), // TODO: add
    Rgb(u8, u8, u8),
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Style {
    pub(crate) fg: Option<Color>,
    pub(crate) bg: Option<Color>,
    pub(crate) bold: Option<bool>,
    pub(crate) intense: Option<bool>,
    pub(crate) underline: Option<bool>,
    pub(crate) italic: Option<bool>,
    pub(crate) reset: Option<bool>,
}

impl Style {
    /// Like `Option::or`: all style values set in `self` are kept, all unset
    /// ones are overwritten with the values from `style_b`.
    pub(crate) fn or(&self, style_b: Self) -> Self {
        Self {
            fg: self.fg.or(style_b.fg),
            bg: self.bg.or(style_b.bg),
            bold: self.bold.or(style_b.bold),
            intense: self.intense.or(style_b.intense),
            underline: self.underline.or(style_b.underline),
            italic: self.italic.or(style_b.italic),
            reset: self.reset.or(style_b.reset),
        }
    }
}