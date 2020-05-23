mod character_sets;
mod nfa;
mod parser;
pub mod rejects;

/// LL(1) CFG for the supported regular expression syntax.
/// https://smlweb.cpsc.ucalgary.ca/vital-stats.php?grammar=UNION+++++-%3E+CONCAT%0D%0A+++++++++++%7C+UNION+cup+CONCAT.%0D%0ACONCAT++++-%3E+UNARY%0D%0A+++++++++++%7C+CONCAT+dot+UNARY.%0D%0AUNARY+++++-%3E+PARegexN+UNARYOP.%0D%0AUNARYOP+++-%3E+*%0D%0A+++++++++++%7C+%3F%0D%0A+++++++++++%7C+%2B%0D%0A+++++++++++%7C.%0D%0APARegexN+++++-%3E+TERM%0D%0A+++++++++++%7C+%28+UNION+%29.%0D%0ATERM++++++-%3E+terminal.%0D%0A
/// grammar before left-recursion is removed (we don't include $ in the grammar because it's just a
/// simple check after parsing)
/*
UNION     -> CONCAT
| UNION cup CONCAT.
CONCAT    -> UNARY
| CONCAT dot UNARY.
UNARY     -> PARegexN UNARYOP.
UNARYOP   -> *
| ?
| +
|.
PARegexN     -> TERM
| ( UNION ).
TERM      -> terminal.
*/
/// Same grammar as above but with left-recursion removed so it is LL(1)
/*
UNION →	CONCAT UNION1 .
UNION1 →	cup CONCAT UNION1
|	.
CONCAT →	UNARY CONCAT1 .
CONCAT1 →	dot UNARY CONCAT1
|	.
UNARY →	PARegexN UNARYOP .
UNARYOP →	*
|	?
|	+
|	.
PARegexN →	TERM
|	( UNION ) .
TERM →	terminal .
*/

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
