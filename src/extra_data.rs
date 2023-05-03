use hcs_lib::data;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ExtraData {}

impl data::Data for ExtraData {}
