#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PagingMode {
    Always,
    QuitIfOneScreen,
    #[default]
    Never,
}
