use std::convert::Infallible;
use std::path::PathBuf;
use std::process::Command;

use super::{Build, Executable};

impl Executable for Build<'_> {
    unsafe fn exec(
        self,
        _argc: i32,
        _argv: *const *const i8,
        _envp: *const *const i8,
    ) -> Result<Infallible, proc_exit::Exit> {
        let mut args = std::env::args_os();
        let exe_filename = args
            .next()
            .map(PathBuf::from)
            .and_then(|path| path.file_name().map(ToOwned::to_owned))
            .unwrap_or_else(|| std::ffi::OsString::from("app.exe"));

        let temp_dir = tempfile::Builder::new()
            .prefix("tg_multivers_")
            .tempdir()
            .map_err(|_| {
                proc_exit::Code::FAILURE.with_message("Failed to create a temporary directory")
            })?;
        let temp_dir_path = temp_dir.path().to_path_buf();

        let exe_path = temp_dir_path.join(&exe_filename);

        let mut file = std::fs::File::create(&exe_path).map_err(|_| {
            proc_exit::Code::FAILURE.with_message("Failed to create the temporary executable")
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = file
                .metadata()
                .map_err(|_| proc_exit::Code::FAILURE.with_message("Failed to read metadata"))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o700);
            std::fs::set_permissions(&exe_path, permissions).unwrap();
        }

        self.extract_into(&mut file).map_err(|_| {
            proc_exit::Code::FAILURE.with_message(format!(
                "Failed to write the build to `{}`",
                exe_path.display()
            ))
        })?;

        // IMPORTANT: Close the file handle so Windows allows it to be executed!
        drop(file);

        // Extract DLLs and decompress them!
        for &(dll_name, dll_bytes) in super::BUNDLED_DLLS {
            let dll_path = temp_dir_path.join(dll_name);
            let mut dll_file = std::fs::File::create(&dll_path).map_err(|_| {
                proc_exit::Code::FAILURE
                    .with_message(format!("Failed to create DLL file `{}`", dll_name))
            })?;
            let mut decoder = bzip2::read::BzDecoder::new(dll_bytes);
            std::io::copy(&mut decoder, &mut dll_file).map_err(|_| {
                proc_exit::Code::FAILURE
                    .with_message(format!("Failed to decompress DLL `{}`", dll_name))
            })?;
        }

        let exit_status = Command::new(&exe_path).args(args).status().map_err(|_| {
            proc_exit::Code::FAILURE.with_message(format!(
                "Failed to execute temporary file `{}`",
                exe_path.display()
            ))
        })?;

        let code = proc_exit::Code::from_status(exit_status);

        // Clean up the temp dir manually before exiting, since process_exit abruptly bypasses Drop/Destructors
        let _ = temp_dir.close();

        code.process_exit()
    }
}
