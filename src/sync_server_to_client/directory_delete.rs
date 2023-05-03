use std::{fs, path};

use hcs_lib::{client_database, data};

pub fn handle_directory_delete(
    file_handler_config: &client_database::FileHandlerConfig,
    directory_delete: data::DirectoryDelete,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(directory_delete.path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    {
        // Delete directory at path
        fs::remove_dir_all(&file_paths.storage_dir_path())?;
    }

    {
        // Delete custom metadata file
        fs::remove_file(&file_paths.custom_metadata_path())?;
    }

    {
        // Delete directory in symlink dir
        fs::remove_dir_all(&file_paths.symlink_dir_path())?;
    }

    Ok(())
}
