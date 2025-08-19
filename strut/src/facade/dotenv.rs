use std::sync::Once;
use strut_core::Pivot;

const FILE_DOT_ENV_LOCAL: &str = ".env.local";
const FILE_DOT_ENV_GLOBAL: &str = ".env";

/// A facade for loading environment variables from `.env` files.
///
/// Strut supports loading variables from `.env.local` and `.env` files located
/// in the application's [pivot directory][pivot].
///
/// Use [`tap`] for a safe, one-time load operation, or [`load`] to perform the
/// operation directly.
///
/// [pivot]: Pivot
pub struct DotEnv;

impl DotEnv {
    /// Ensures environment variables from dot-env files are loaded.
    ///
    /// This function guarantees that the loading operation is performed at most
    /// once during the application's lifecycle. Subsequent calls will have no
    /// effect.
    ///
    /// This is the recommended method for applying dot-env configuration in most
    /// scenarios. It internally calls [`load`] on its first invocation.
    ///
    /// [`load`]: DotEnv::load
    pub fn tap() {
        static INIT: Once = Once::new();

        INIT.call_once(Self::load);
    }

    /// Loads environment variables from dot-env files into the environment.
    ///
    /// This method does not override any environment variables that are already
    /// set. It only loads values for variables that are not currently present in
    /// the process's environment.
    ///
    /// ## Precedence
    ///
    /// The files are loaded in the following order, with variables from earlier
    /// files taking precedence:
    ///
    /// 1. `.env.local`
    /// 2. `.env`
    ///
    /// Both files are searched for in the application's [pivot directory][pivot].
    /// If a file is not found, it is silently ignored.
    ///
    /// [pivot]: Pivot
    pub fn load() {
        let pivot = Pivot::resolve();

        // Load local file (first priority)
        let _ = dotenvy::from_path(pivot.join(FILE_DOT_ENV_LOCAL));

        // Load global file (second priority)
        let _ = dotenvy::from_path(pivot.join(FILE_DOT_ENV_GLOBAL));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use scopeguard::defer;
    use std::env::remove_var;
    use std::fs::{remove_file, File};
    use std::io::Write;

    const TEST_VARIABLE_ENV_LOC_GLO: &str = "TEST_VARIABLE_ENV_LOC_GLO";
    const TEST_VARIABLE_ENV_LOC____: &str = "TEST_VARIABLE_ENV_LOC____";
    const TEST_VARIABLE_ENV_____GLO: &str = "TEST_VARIABLE_ENV_____GLO";
    const TEST_VARIABLE_ENV________: &str = "TEST_VARIABLE_ENV________";
    const TEST_VARIABLE_____LOC_GLO: &str = "TEST_VARIABLE_____LOC_GLO";
    const TEST_VARIABLE_____LOC____: &str = "TEST_VARIABLE_____LOC____";
    const TEST_VARIABLE_________GLO: &str = "TEST_VARIABLE_________GLO";
    const TEST_VARIABLE____________: &str = "TEST_VARIABLE____________";

    #[test]
    fn test_dotenv_tap() {
        // Set up the initial state
        unsafe {
            std::env::set_var(TEST_VARIABLE_ENV_LOC_GLO, "env");
            std::env::set_var(TEST_VARIABLE_ENV_LOC____, "env");
            std::env::set_var(TEST_VARIABLE_ENV_____GLO, "env");
            std::env::set_var(TEST_VARIABLE_ENV________, "env");
        }
        create_dotenv_files("loc", "glo");

        // Ensure cleanup is executed after the test, even on failure
        defer! {
            clean_up_files();
            clean_up_environment();
        }

        // Check values in initial environment
        assert(TEST_VARIABLE_ENV_LOC_GLO, "env");
        assert(TEST_VARIABLE_ENV_LOC____, "env");
        assert(TEST_VARIABLE_ENV_____GLO, "env");
        assert(TEST_VARIABLE_ENV________, "env");
        assert(TEST_VARIABLE_____LOC_GLO, "");
        assert(TEST_VARIABLE_____LOC____, "");
        assert(TEST_VARIABLE_________GLO, "");
        assert(TEST_VARIABLE____________, "");

        // Tap the dot-env files
        DotEnv::tap();

        // Check values in updated environment
        assert(TEST_VARIABLE_ENV_LOC_GLO, "env");
        assert(TEST_VARIABLE_ENV_LOC____, "env");
        assert(TEST_VARIABLE_ENV_____GLO, "env");
        assert(TEST_VARIABLE_ENV________, "env");
        assert(TEST_VARIABLE_____LOC_GLO, "loc");
        assert(TEST_VARIABLE_____LOC____, "loc");
        assert(TEST_VARIABLE_________GLO, "glo");
        assert(TEST_VARIABLE____________, "");

        // Tap the dot-env files again (should have no additional effect)
        DotEnv::tap();

        // Check values in updated environment
        assert(TEST_VARIABLE_ENV_LOC_GLO, "env");
        assert(TEST_VARIABLE_ENV_LOC____, "env");
        assert(TEST_VARIABLE_ENV_____GLO, "env");
        assert(TEST_VARIABLE_ENV________, "env");
        assert(TEST_VARIABLE_____LOC_GLO, "loc");
        assert(TEST_VARIABLE_____LOC____, "loc");
        assert(TEST_VARIABLE_________GLO, "glo");
        assert(TEST_VARIABLE____________, "");

        // Re-create the state with different values
        unsafe {
            std::env::set_var(TEST_VARIABLE_ENV_LOC_GLO, "new_env");
            std::env::set_var(TEST_VARIABLE_ENV_LOC____, "new_env");
            std::env::set_var(TEST_VARIABLE_ENV_____GLO, "new_env");
            std::env::set_var(TEST_VARIABLE_ENV________, "new_env");
        }
        clean_up_files();
        create_dotenv_files("new_loc", "new_glo");

        // Tap the dot-env files again (should have no additional effect)
        DotEnv::tap();

        // Check values in updated environment
        assert(TEST_VARIABLE_ENV_LOC_GLO, "new_env");
        assert(TEST_VARIABLE_ENV_LOC____, "new_env");
        assert(TEST_VARIABLE_ENV_____GLO, "new_env");
        assert(TEST_VARIABLE_ENV________, "new_env");
        assert(TEST_VARIABLE_____LOC_GLO, "loc");
        assert(TEST_VARIABLE_____LOC____, "loc");
        assert(TEST_VARIABLE_________GLO, "glo");
        assert(TEST_VARIABLE____________, "");
    }

    fn create_dotenv_files(local_value: &str, global_value: &str) {
        // Create `.env.local`
        let mut local_file: File = File::create(FILE_DOT_ENV_LOCAL)
            .unwrap_or_else(|_| panic!("it should be possible to create {}", FILE_DOT_ENV_LOCAL));
        write_to_dotenv_file(
            &mut local_file,
            FILE_DOT_ENV_LOCAL,
            TEST_VARIABLE_ENV_LOC_GLO,
            local_value,
        );
        write_to_dotenv_file(
            &mut local_file,
            FILE_DOT_ENV_LOCAL,
            TEST_VARIABLE_ENV_LOC____,
            local_value,
        );
        write_to_dotenv_file(
            &mut local_file,
            FILE_DOT_ENV_LOCAL,
            TEST_VARIABLE_____LOC_GLO,
            local_value,
        );
        write_to_dotenv_file(
            &mut local_file,
            FILE_DOT_ENV_LOCAL,
            TEST_VARIABLE_____LOC____,
            local_value,
        );

        // Create `.env` if a variable is provided
        let mut global_file: File = File::create(FILE_DOT_ENV_GLOBAL)
            .unwrap_or_else(|_| panic!("it should be possible to create {}", FILE_DOT_ENV_GLOBAL));
        write_to_dotenv_file(
            &mut global_file,
            FILE_DOT_ENV_GLOBAL,
            TEST_VARIABLE_ENV_LOC_GLO,
            global_value,
        );
        write_to_dotenv_file(
            &mut global_file,
            FILE_DOT_ENV_GLOBAL,
            TEST_VARIABLE_ENV_____GLO,
            global_value,
        );
        write_to_dotenv_file(
            &mut global_file,
            FILE_DOT_ENV_GLOBAL,
            TEST_VARIABLE_____LOC_GLO,
            global_value,
        );
        write_to_dotenv_file(
            &mut global_file,
            FILE_DOT_ENV_GLOBAL,
            TEST_VARIABLE_________GLO,
            global_value,
        );
    }

    fn write_to_dotenv_file(
        file: &mut File,
        file_name: &str,
        env_var_name: &str,
        env_var_value: &str,
    ) {
        writeln!(file, "{}={}", env_var_name, env_var_value)
            .unwrap_or_else(|_| panic!("it should be possible to write to {}", file_name));
    }

    fn clean_up_files() {
        let _ = remove_file(FILE_DOT_ENV_LOCAL);
        let _ = remove_file(FILE_DOT_ENV_GLOBAL);
    }

    fn clean_up_environment() {
        unsafe {
            remove_var(TEST_VARIABLE_ENV_LOC_GLO);
            remove_var(TEST_VARIABLE_ENV_LOC____);
            remove_var(TEST_VARIABLE_ENV_____GLO);
            remove_var(TEST_VARIABLE_ENV________);
            remove_var(TEST_VARIABLE_____LOC_GLO);
            remove_var(TEST_VARIABLE_____LOC____);
            remove_var(TEST_VARIABLE_________GLO);
        }
    }

    fn assert(name: &str, expected: &str) {
        let actual = std::env::var(name).unwrap_or_else(|_| "".to_string());

        assert_eq!(
            expected, &actual,
            "environment variable {} is expected to be set to '{}' but is instead set to '{}'",
            name, expected, &actual,
        );
    }
}
