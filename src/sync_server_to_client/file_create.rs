use std::{fs, io::Write, path};

use hcs_lib::{client_database, data, protocol};

use crate::bytes_to_transmission_type;

pub fn handle_file_create(
    tcp_connection: &mut Box<protocol::TcpConnection>,
    file_handler_config: &client_database::FileHandlerConfig,
    file_create: data::FileCreate,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(file_create.path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    {
        // Read file from server and write to location
        let mut file = fs::File::create(&file_paths.storage_dir_path())?;
        let packets = protocol::calculate_num_packets(file_create.size());
        for _ in 0..packets {
            let bytes = tcp_connection.read_next_chunk()?;

            match bytes_to_transmission_type(&bytes) {
                Ok(transmission) => match transmission {
                    data::Transmission::SkipCurrent => {
                        return Ok(());
                    }
                    _ => {
                        return Err("Expected no transmission, got something one".into());
                    }
                },
                Err(_) => {}
            };

            file.write(&bytes)?;
        }
    }

    {
        // Create custom metadata file
        let last_modified =
            client_database::CustomMetadata::last_modified_of_file(file_paths.storage_dir_path())?;
        let custom_metadata = client_database::CustomMetadata::new(last_modified);
        custom_metadata.write_to_file(&file_paths)?;
    }

    {
        // create symlink to file
        symlink::symlink_file(
            &file_paths.storage_dir_path(),
            &file_paths.symlink_dir_path(),
        )?;
    }

    Ok(())
}