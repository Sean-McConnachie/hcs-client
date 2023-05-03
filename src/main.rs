use hcs_client::{args, config};
use hcs_lib::logger;

fn main() {
    let config: config::ClientConfig =
        hcs_lib::config::read_config("Config.toml").expect("Failed to read config file");

    logger::init_logger(config.log_level());

    args::run_from_args(&config).unwrap();
}
