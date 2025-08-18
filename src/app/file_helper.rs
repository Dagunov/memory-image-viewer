use std::{cell::RefCell, path::PathBuf, rc::Rc};

use super::logger::Logger;

pub enum FileDialogMode {
    SaveFile,
    SelectFolder,
}

pub enum FileDialogError {
    NoImplementation,
    // dont really care for other errors
    Other,
}

pub fn file_dialog(
    fd_mode: FileDialogMode,
    logger: &Rc<RefCell<Logger>>,
) -> Result<PathBuf, FileDialogError> {
    let fd = native_dialog::FileDialog::new();
    let fd_res = match fd_mode {
        FileDialogMode::SaveFile => fd.add_filter("png", &["png"]).show_save_single_file(),
        FileDialogMode::SelectFolder => fd.show_open_single_dir(),
    };
    match fd_res {
        Ok(path) => {
            if let Some(path) = path {
                Ok(path)
            } else {
                logger.borrow_mut().info("Out location not selected");
                Err(FileDialogError::Other)
            }
        }
        Err(native_dialog::Error::NoImplementation) => {
            match fd_mode {
                FileDialogMode::SaveFile => {
                    logger
                        .borrow_mut()
                        .error("File dialog not supported on your OS, save file via \"Dump...\"");
                }
                FileDialogMode::SelectFolder => {
                    logger.borrow_mut().error(
                        "File dialog not supported on your OS, enter path manually in settings",
                    );
                }
            };
            Err(FileDialogError::NoImplementation)
        }
        Err(e) => {
            logger
                .borrow_mut()
                .error("File dialog error, enter path manually in settings or try again");
            logger
                .borrow_mut()
                .error(format!("FileDialog error: {:?}", e));
            Err(FileDialogError::Other)
        }
    }
}
