use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use rejects_macro::make_rejects;

pub mod builder;
mod character_sets;
pub mod nfa;
mod parser;
pub mod rejects;

/// Rejects is an implementation of regular expressions that implements the following:
///     '*': Zero or more on the preceding (based on operator precedence) regular expression.
///     '+': One or more on the preceding (based on operator precedence) regular expression.
///     '?': Zero or One on the preceding (based on operator precedence) regular expression.
///     '|': For union of multiple regular expressions.
///     '()': For precedence only, referring to capture groups with \1 is not yet supported.
///     '[]': For union of various characters, character ranges over ascii characters (e.g. a-z, 0-9, A-Z),
///           character sets ('\w', '\W', '\d', '\D', '\s', '\S'), '\' is supported by escaping it ('\\').
///           The entire block can be negated using '^' at the start (e.g. [^a-z] to match anything
///           except [a-z]).
///     '.': Any character.
///     '\': Denotes the following character to be special. Special characters are members of the
///          set {'w', 'W', 'd', 'D', 's', 'S', '*', '+', '\', '(', ')', '.'}. They work as
///          expected based on PCRE2.
///
/// The grammar is explained in parser.rs.

#[cfg(test)]
mod tests {
    use crate::parser;

    #[test]
    fn test_valid_regexes() {
        let regexes = vec![
            r"abcd",
            r"a|b|c|d",
            r"(ab)|(cd)*",
            r"(a|b+c?|d)",
            r"(abcd)",
            r"a|a",
            r"(bc)",
            r"(abc)(abc)(abc)|(abc)(abc)",
            r"a|b+(c?|d)",
            r"(a|b)",
            r"(\w)",
            r"\\",
            r"[1-9]",
        ];
        for regex in regexes {
            assert!(
                parser::parse(regex).is_ok(),
                "\"{}\" should be recognized as valid regex",
                regex
            );
        }
    }

    #[test]
    fn test_invalid_regexes() {
        let regexes = vec![
            r"(abcd",
            r"a||c|d",
            r"|",
            r"()",
            r")aaab(",
            r"a|b+c?|d)",
            r"a)",
            r"(abcd)(",
            r"(a|)",
            r"(\a)",
            r"\",
            r"\\\",
        ];
        for regex in regexes.iter() {
            assert!(
                parser::parse(regex).is_err(),
                r#""{}" should be recognized as an invalid regex"#,
                regex
            );
        }
    }

    #[test]
    fn test_union() {}

    #[test]
    fn test_concatenation() {}
}
