use hcs_lib::{client_database, config};

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ClientConfig {
    #[serde(deserialize_with = "config::parse_log_filter")]
    log_level: log::LevelFilter,

    tcp_config: TcpConfig,
    file_handler_config: client_database::FileHandlerConfig,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TcpConfig {
    addr: String,
}

impl ClientConfig {
    pub fn log_level(&self) -> log::LevelFilter {
        self.log_level
    }

    pub fn tcp_addr(&self) -> &str {
        &self.tcp_config.addr
    }

    pub fn file_handler_config(&self) -> &client_database::FileHandlerConfig {
        &self.file_handler_config
    }
}

impl TcpConfig {
    pub fn addr(&self) -> &str {
        &self.addr
    }
}
