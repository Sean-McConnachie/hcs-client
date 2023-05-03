use hcs_lib::data;

pub mod args;
pub mod config;
pub mod errors;
pub mod extra_data;
pub mod sync_client_to_server;
pub mod sync_server_to_client;

fn bytes_to_transmission_type(
    bytes: &[u8],
) -> Result<
    data::Transmission<errors::ServerTcpError, extra_data::ExtraData>,
    Box<dyn std::error::Error>,
> {
    let return_type: data::Transmission<errors::ServerTcpError, extra_data::ExtraData> =
        bincode::deserialize(bytes)?;
    Ok(return_type)
}

pub fn transmission_type_to_bytes(
    transmission: data::Transmission<errors::ServerTcpError, extra_data::ExtraData>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = bincode::serialize(&transmission)?;
    Ok(bytes)
}
