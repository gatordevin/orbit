use std::path::PathBuf;

use crate::Command;
use crate::FromCli;
use crate::core::manifest::IpManifest;
use crate::interface::cli::Cli;
use crate::interface::arg::Positional;
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_BLUEPRINT;
use crate::util::environment::ORBIT_WIN_LITERAL_CMD;
use crate::util::filesystem;

use super::plan::BLUEPRINT_FILE;

#[derive(Debug, PartialEq)]
pub struct Env {
    keys: Vec<String>,
}

impl FromCli for Env {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        // collect all positional arguments
        let mut keys: Vec<String> = Vec::new();
        while let Some(c) = cli.check_positional(Positional::new("key"))? {
            keys.push(c);
        }
        let command = Ok(Env {
            keys: keys,
        });
        command
    }
}

impl Command for Env {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // assemble environment information
        let mut env = Environment::from_vec(vec![
            // @todo: context should own an `Environment` struct instead of this data transformation
            EnvVar::new().key(environment::ORBIT_HOME).value(filesystem::normalize_path(c.get_home_path().clone()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_CACHE).value(filesystem::normalize_path(c.get_cache_path().to_path_buf()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_BUILD_DIR).value(c.get_build_dir()),
            EnvVar::new().key(environment::ORBIT_DEV_PATH).value(filesystem::normalize_path(c.get_development_path().unwrap_or(&PathBuf::new()).clone()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_IP_PATH).value(filesystem::normalize_path(c.get_ip_path().unwrap_or(&PathBuf::new()).clone()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_STORE).value(filesystem::normalize_path(c.get_store_path().clone()).to_str().unwrap()),
            EnvVar::new().key("EDITOR").value(&std::env::var("EDITOR").unwrap_or(String::new())),
            EnvVar::new().key("NO_COLOR").value(&std::env::var("NO_COLOR").unwrap_or(String::new())),
            ])
            .from_config(c.get_config())?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE));

        // add platform-specific environment variables
        if cfg!(target_os = "windows") {
            env = env.add(EnvVar::new().key(ORBIT_WIN_LITERAL_CMD).value(&std::env::var(ORBIT_WIN_LITERAL_CMD).unwrap_or(String::new())));
        }

        // check if in an ip to add those variables
        if c.goto_ip_path().is_ok() {
            // check ip
            if let Ok(ip) = IpManifest::from_path(c.get_ip_path().unwrap()) {
                env = env.from_ip(&ip)?;
            }
            // check the build directory
            env = env.from_env_file( &std::path::PathBuf::from(c.get_build_dir()))?;
        }
        
        self.run(env)
    }
}

impl Env {
    fn run(&self, env: Environment) -> Result<(), Box<dyn std::error::Error>> {
        let mut result = String::new();

        match self.keys.is_empty() {
            // print debugging output (all variables)
            true => {
                env.iter().for_each(|e| {
                    if result.is_empty() == false {
                        result.push('\n');
                    }
                    result.push_str(&format!("{:?}", e))
                });
            },
            false => {
                let mut initial = true;
                // print values only
                self.keys.iter().for_each(|k| {
                    if initial == false {
                        result.push('\n');
                    }
                    if let Some(entry) = env.get(k) {
                        result.push_str(&entry.get_value());
                    }
                    initial = false;
                });
            }
        }

        println!("{}", result);
        Ok(())
    }
}

const HELP: &str = "\
Display Orbit environment information.

Usage:
    orbit env [options]

Options:
    <key>...     A environment variable to display its value

Use 'orbit help env' to learn more about the command.
";