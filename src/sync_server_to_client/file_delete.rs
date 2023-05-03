use std::{fs, path};

use hcs_lib::{client_database, data};

pub fn handle_file_delete(
    file_handler_config: &client_database::FileHandlerConfig,
    file_delete: data::FileDelete,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(file_delete.path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    {
        // Delete file
        fs::remove_file(&file_paths.storage_dir_path())?;
    }

    {
        // Delete custom metadata file
        fs::remove_file(&file_paths.custom_metadata_path())?;
    }

    {
        // Delete symlink
        symlink::remove_symlink_file(&file_paths.symlink_dir_path())?;
    }

    Ok(())
}
