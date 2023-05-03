use std::{fs, net};

use hcs_lib::{client_database, data, protocol};

mod file_create;
mod file_modify;

use crate::{bytes_to_transmission_type, config, errors, extra_data, transmission_type_to_bytes};

pub fn sync_client_to_server(
    config: &config::ClientConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_version =
        client_database::ServerVersion::init(&config.file_handler_config().program_data_directory);

    start_transmission(
        net::TcpStream::connect(config.tcp_addr())?,
        &config.file_handler_config(),
        server_version,
    )?;
    Ok(())
}

fn start_transmission(
    tcp_stream: net::TcpStream,
    file_handler_config: &client_database::FileHandlerConfig,
    mut server_version: client_database::ServerVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Starting sync client to server transmission");
    let mut tcp_connection = protocol::TcpConnection::new(tcp_stream);

    {
        log::debug!("Sending greeting");
        // Send greeting to server.
        let greeting = data::Greeting::new("HCS CLIENT".to_string());
        let transmission = data::Transmission::Greeting(greeting);
        let bytes = transmission_type_to_bytes(transmission)?;
        tcp_connection.write(&bytes)?;
    }

    {
        log::debug!("Waiting for server to respond with proceed");
        // Ensure the server is ready to proceed.
        let response = tcp_connection.read_next_chunk()?;
        let transmission = bytes_to_transmission_type(&response)?;
        if transmission != data::Transmission::Proceed {
            log::error!("Server did not respond with proceed");
            return Err("Server did not respond with proceed".into());
        }
    }

    let changes = {
        let changes = client_database::read_changes(file_handler_config);
        let optimized_changes = data::optimize_changes(changes);
        optimized_changes
    };

    log::debug!("{} changes to send", changes.len());

    {
        log::debug!("Sending SyncClientToServer");
        // Send SyncClientToServer
        let sync_client_to_server =
            data::SyncClientToServer::new(server_version.server_version(), changes.len() as i32);
        let transmission = data::Transmission::SyncClientToServer(sync_client_to_server);
        let bytes = transmission_type_to_bytes(transmission)?;
        tcp_connection.write(&bytes)?;
    }

    {
        log::debug!("Waiting for server to respond with proceed");
        // Ensure the server is ready to proceed
        // `ServerVersion` response means the client must perform `SyncServerToClient` first
        // `Proceed` response means the client can perform `SyncClientToServer`
        let response = tcp_connection.read_next_chunk()?;
        let transmission = bytes_to_transmission_type(&response)?;
        match transmission {
            data::Transmission::ServerVersion(_) => {
                log::error!("Server responded with ServerVersion. You must first sync the server to the client.");
                return Err("Server responded with ServerVersion. You must first sync the server to the client.".into());
            }
            data::Transmission::Proceed => {}
            _ => {
                log::error!("Server did not respond with proceed");
                return Err("Server did not respond with proceed".into());
            }
        }
    }

    {
        let changes_len = changes.len();
        // Loop over `SyncClientToServer` num_changes()
        for (change_num, change) in changes.into_iter().enumerate() {
            log::info!("Sending change {} of {}", change_num + 1, changes_len);
            let cloned_change = {
                let update_change = match change.1 {
                    data::ChangeEvent::File(data::FileEvent::Create(mut file_create)) => {
                        let file_size = fs::metadata(
                            file_handler_config
                                .storage_directory
                                .join(&file_create.path()),
                        )?
                        .len();
                        file_create.set_size(file_size);
                        data::ChangeEvent::File(data::FileEvent::Create(file_create))
                    }
                    data::ChangeEvent::File(data::FileEvent::Modify(mut file_modify)) => {
                        let file_size = fs::metadata(
                            file_handler_config
                                .storage_directory
                                .join(&file_modify.path()),
                        )?
                        .len();
                        file_modify.set_size(file_size);
                        data::ChangeEvent::File(data::FileEvent::Modify(file_modify))
                    }
                    _ => change.1,
                };
                let cloned_change = update_change.clone();
                // Send the ChangeEvent as a Transmission to the server.
                let transmission = data::Transmission::<
                    errors::ServerTcpError,
                    extra_data::ExtraData,
                >::ChangeEvent(update_change);
                let bytes = transmission_type_to_bytes(transmission)?;
                tcp_connection.write(&bytes)?;
                cloned_change
            };

            match cloned_change {
                data::ChangeEvent::File(file_event) => match file_event {
                    data::FileEvent::Create(file_create) => {
                        file_create::handle_file_create(
                            &mut tcp_connection,
                            &file_handler_config,
                            file_create,
                        )?;
                    }
                    data::FileEvent::Modify(file_modify) => {
                        file_modify::handle_file_modify(
                            &mut tcp_connection,
                            &file_handler_config,
                            file_modify,
                        )?;
                    }
                    _ => {}
                },
                _ => {}
            }

            {
                log::debug!("Waiting for server to respond with new version.");
                // get new server version and delete change file.
                let bytes = tcp_connection.read_next_chunk()?;
                let transmission = bytes_to_transmission_type(&bytes)?;
                let sv = match transmission {
                    data::Transmission::ServerVersion(sv) => sv,
                    _ => {
                        log::error!("Server did not respond with ServerVersion");
                        return Err("Server did not respond with ServerVersion".into());
                    }
                };
                server_version.set(sv.server_version());

                // delete change file
                let change_path = file_handler_config
                    .program_data_directory
                    .join("changes")
                    .join(format!("{}.tmp", change.0));
                fs::remove_file(change_path)?;
            }
        }
    }

    Ok(())
}
