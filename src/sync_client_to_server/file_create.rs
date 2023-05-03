use std::{fs, io::Read};

use hcs_lib::{client_database, data, protocol};

pub fn handle_file_create(
    tcp_connection: &mut Box<protocol::TcpConnection>,
    file_handler_config: &client_database::FileHandlerConfig,
    file_create: data::FileCreate,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file buffer by buffer, write into tcp stream.
    let file_path = file_handler_config
        .storage_directory
        .join(file_create.path());
    let file_size = file_create.size();

    let packets = protocol::calculate_num_packets(file_size);
    let mut file = fs::File::open(&file_path)?;
    let mut buffer = vec![0; protocol::BUFFER_SIZE];
    for _ in 0..packets {
        let bytes_read = file.read(&mut buffer)?;
        tcp_connection.write(&buffer[..bytes_read])?;
    }

    Ok(())
}
