use std::{fs, path};

use hcs_lib::{client_database, data};

pub fn handle_file_move(
    file_handler_config: &client_database::FileHandlerConfig,
    file_move: data::FileMove,
) -> Result<(), Box<dyn std::error::Error>> {
    let from_file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(file_move.from_path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    let to_file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(file_move.to_path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    {
        // Move file
        fs::rename(
            &from_file_paths.storage_dir_path(),
            &to_file_paths.storage_dir_path(),
        )?;
    }

    {
        // Move custom metadata file
        fs::rename(
            &from_file_paths.custom_metadata_path(),
            &to_file_paths.custom_metadata_path(),
        )?;
    }

    {
        // Delete symlink, then create a new one
        symlink::remove_symlink_file(&from_file_paths.symlink_dir_path())?;
        symlink::symlink_file(
            &to_file_paths.storage_dir_path(),
            &to_file_paths.symlink_dir_path(),
        )?;
    }

    Ok(())
}
