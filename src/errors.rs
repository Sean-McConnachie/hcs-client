use hcs_lib::data;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ServerTcpError {}

impl data::Data for ServerTcpError {}
