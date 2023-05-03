use std::{fs, path};

use hcs_lib::{client_database, data};

pub fn handle_directory_create(
    file_handler_config: &client_database::FileHandlerConfig,
    directory_create: data::DirectoryCreate,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_paths = client_database::FilePaths::from_relative_path(
        path::PathBuf::from(directory_create.path()),
        client_database::Type::File,
        client_database::FileLocation::StorageDir,
        None,
        file_handler_config,
    )?;

    {
        // Create directory at path
        fs::create_dir_all(&file_paths.storage_dir_path())?;
    }

    {
        // Create custom metadata file
        let last_modified =
            client_database::CustomMetadata::last_modified_of_file(&file_paths.storage_dir_path())?;
        let custom_metadata = client_database::CustomMetadata::new(last_modified);
        custom_metadata.write_to_file(&file_paths)?;
    }

    {
        // Create directory in symlink dir
        fs::create_dir_all(&file_paths.symlink_dir_path())?;
    }

    Ok(())
}
