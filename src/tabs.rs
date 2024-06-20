use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, FromRepr};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCount, PartialEq, Debug)]
pub enum Tab {
  #[default]
  Search,
  Replace,
  SearchResult,
  Preview,
}
