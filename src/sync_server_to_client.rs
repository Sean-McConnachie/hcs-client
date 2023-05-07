use std::net;

use hcs_lib::{client_database, data, protocol};

use crate::{bytes_to_transmission_type, config, transmission_type_to_bytes};

mod directory_create;
mod directory_delete;
mod directory_move;
mod file_create;
mod file_delete;
mod file_modify;
mod file_move;

pub fn sync_server_to_client(
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

fn handle_server_to_client_change_event(
    tcp_connection: &mut Box<protocol::TcpConnection>,
    file_handler_config: &client_database::FileHandlerConfig,
    change_event: data::ChangeEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        // handle that change event
        match change_event {
            data::ChangeEvent::File(file_event) => match file_event {
                data::FileEvent::Create(file_create) => {
                    file_create::handle_file_create(
                        tcp_connection,
                        file_handler_config,
                        file_create,
                    )?;
                }
                data::FileEvent::Modify(file_modify) => {
                    file_modify::handle_file_modify(
                        tcp_connection,
                        file_handler_config,
                        file_modify,
                    )?;
                }
                data::FileEvent::Delete(file_delete) => {
                    file_delete::handle_file_delete(file_handler_config, file_delete)?;
                }
                data::FileEvent::Move(file_move) => {
                    file_move::handle_file_move(file_handler_config, file_move)?;
                }
                data::FileEvent::UndoDelete(_file_undo_delete) => {
                    unimplemented!("Undo file delete")
                }
            },
            data::ChangeEvent::Directory(directory_event) => match directory_event {
                data::DirectoryEvent::Create(directory_create) => {
                    directory_create::handle_directory_create(
                        file_handler_config,
                        directory_create,
                    )?;
                }
                data::DirectoryEvent::Delete(directory_delete) => {
                    directory_delete::handle_directory_delete(
                        file_handler_config,
                        directory_delete,
                    )?;
                }
                data::DirectoryEvent::Move(directory_move) => {
                    directory_move::handle_directory_move(file_handler_config, directory_move)?;
                }
                data::DirectoryEvent::UndoDelete(_directory_undo_delete) => {
                    unimplemented!("Undo directory delete")
                }
            },
            data::ChangeEvent::Symlink(_) => unimplemented!("Symlink not implemented"),
        }
    }
    Ok(())
}

fn start_transmission(
    tcp_stream: net::TcpStream,
    file_handler_config: &client_database::FileHandlerConfig,
    mut server_version: client_database::ServerVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Starting sync server to client transmission");
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

    {
        log::debug!("Sending SyncServerToClient");
        // Send SyncServerToClient
        let sync_server_to_client = data::SyncServerToClient::new(server_version.server_version());
        let transmission = data::Transmission::SyncServerToClient(sync_server_to_client);
        let bytes = transmission_type_to_bytes(transmission)?;
        tcp_connection.write(&bytes)?;
    }

    loop {
        log::info!("Waiting for change event");
        {
            let bytes = tcp_connection.read_next_chunk()?;
            let transmission = bytes_to_transmission_type(&bytes)?;
            dbg!(&transmission);
            match transmission {
                data::Transmission::ChangeEvent(change_event) => {
                    handle_server_to_client_change_event(
                        &mut tcp_connection,
                        file_handler_config,
                        change_event.clone(),
                    )?;
                }
                data::Transmission::SkipCurrent => {
                    log::info!("Server sent skip current event.");
                }
                data::Transmission::TransactionComplete => {
                    log::info!("Server sent transaction complete event.");
                    return Ok(());
                }
                data::Transmission::ServerVersion(sv) => {
                    log::info!("Server sent server version event.");
                    server_version.set(sv.server_version());
                }
                _ => {
                    log::error!("Server did not respond with change event");
                    return Err("Server did not respond with change event".into());
                }
            }
        };

        {
            // get either `ServerVersion` or `TransactionComplete`
            log::debug!("Waiting for server to respond with server version");
            let bytes = tcp_connection.read_next_chunk()?;
            let transmission = bytes_to_transmission_type(&bytes)?;
            match transmission {
                data::Transmission::ServerVersion(server_version_response) => {
                    log::info!(
                        "Server version: {}",
                        server_version_response.server_version()
                    );
                    server_version.set(server_version_response.server_version());
                }
                data::Transmission::TransactionComplete => {
                    log::info!("Server sent transaction complete event.");
                    return Ok(());
                }

                _ => {
                    dbg!(transmission);
                    log::error!("Server did not respond with server version");
                    return Err("Server did not respond with server version".into());
                }
            }
        }
    }
}
