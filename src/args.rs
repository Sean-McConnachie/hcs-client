use std::env;

use hcs_lib::client_detect_offline;

use crate::{config, sync_client_to_server, sync_server_to_client};

pub fn run_from_args(config: &config::ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        args.push("".to_string())
    }
    match (&*args[1], &*args[2]) {
        ("detect", _) => {
            client_detect_offline::detect_offline_changes(&config.file_handler_config())
        }
        ("sync", "up") => {
            client_detect_offline::detect_offline_changes(&config.file_handler_config());
            sync_client_to_server::sync_client_to_server(&config).unwrap();
        }
        ("sync", "down") => {
            client_detect_offline::detect_offline_changes(&config.file_handler_config());
            sync_server_to_client::sync_server_to_client(&config).unwrap();
        }
        ("sync", _) => {
            client_detect_offline::detect_offline_changes(&config.file_handler_config());
            sync_server_to_client::sync_server_to_client(&config).unwrap();
            sync_client_to_server::sync_client_to_server(&config).unwrap();
        }
        ("live", _) => {
            // detect_offline_changes(&config);
            // sync_client_to_server(&config);
            // sync_server_to_client(&config);
            // run_live(&config);
            unimplemented!("Live mode is not yet implemented.")
        }
        ("help", _) => {
            println!(
                "hcs detect\t- Detects any changes that were made while the program was offline."
            );
            println!("hcs live\t- (UNIMPLEMENTED)\tDetects, then watches the `shortcut` directory for changes. Periodically syncs to and from server.");
            println!("hcs sync up\t- Detects, then syncs local changes to the server.");
            println!("hcs sync down\t- Detects, then syncs server changes to client");
            println!("hcs sync\t- Detects, then syncs up, then syncs down");
        }
        _ => (),
    }

    Ok(())
}
