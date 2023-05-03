#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

/// How to print non-printable characters with
/// [crate::config::Config::show_nonprintable]
#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NonprintableNotation {
    /// Use caret notation (^G, ^J, ^@, ..)
    Caret,

    /// Use unicode notation (␇, ␊, ␀, ..)
    #[default]
    Unicode,
}
