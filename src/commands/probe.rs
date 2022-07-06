use std::collections::BTreeMap;

use crate::Command;
use crate::FromCli;
use crate::core::manifest::IpManifest;
use crate::core::pkgid::PkgId;
use crate::core::version::AnyVersion;
use crate::core::version::Version;
use crate::core::vhdl::primaryunit::PrimaryUnit;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;

use super::search;

#[derive(Debug, PartialEq)]
pub struct Probe {
    ip: PkgId,
    tags: bool,
    units: bool,
    version: Option<AnyVersion>,
    changelog: bool,
    readme: bool,
}

impl FromCli for Probe {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Probe {
            tags: cli.check_flag(Flag::new("tags"))?,
            units: cli.check_flag(Flag::new("units"))?,
            changelog: cli.check_flag(Flag::new("changes"))?,
            readme: cli.check_flag(Flag::new("readme"))?,
            version: cli.check_option(Optional::new("ver").switch('v'))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        });
        command
    }
}

impl Command for Probe {
    type Err = Fault;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // collect all manifests
        let mut universe = search::Search::all_pkgid((
            c.get_development_path().unwrap(), 
            c.get_cache_path(), 
            &c.get_vendor_path()))?;
        let ids: Vec<&PkgId> = universe.keys().into_iter().collect();
        let target = crate::core::ip::find_ip(&self.ip, ids)?;

        // ips under this key
        let inventory = universe.remove(&target).unwrap();

        let dev_ver = match &inventory.0 {
            Some(ip) => Some(ip.into_version()),
            None => None,
        };

        let inst_ver: Vec<Version> = inventory.1.iter().map(|f| f.into_version()).collect();
        let avl_ver: Vec<Version> = inventory.2.iter().map(|f| f.into_version()).collect();
        
        // collect all ip in the user's universe to see if ip exists
        if self.tags == true {
            println!("{}", format_version_table((dev_ver, inst_ver, avl_ver)));
            return Ok(())
        }

        // find most compatible version with the partial version
        let v = self.version.as_ref().unwrap_or(&AnyVersion::Latest);
        let ip = match v {
            AnyVersion::Dev => {
                // take the manifest from the DEV_PATH
                match inventory.0 {
                    Some(i) => Ip::from_manifest(i),
                    None => return Err(AnyError(format!("ip '{}' is not found on the development path", target)))?
                }
            },
            _ => select_ip_from_version(&target, &v, inventory.1)?
        };

        if self.units == true {
            let units = ip.collect_units();
            println!("{}", format_units_table(units));
            return Ok(())
        }

        println!("{}", ip.into_manifest());

        self.run()
    }
}

impl Probe {
    fn run(&self) -> Result<(), Fault> {
        Ok(())
    }
}

/// Creates an IP from an ID `target` and version `v` within the given `inventory` of manifests.
pub fn select_ip_from_version(target: &PkgId, v: &AnyVersion, inventory: Vec<IpManifest>) -> Result<Ip, Fault>  {
    let inst_ver: Vec<Version> = inventory.iter().map(|f| f.into_version()).collect();
    let version = crate::commands::install::get_target_version(v, &inst_ver, &target)?;
    let ip = inventory.into_iter().find(|f| &f.into_version() == version).unwrap();
    Ok(Ip::from_manifest(ip))
}

/// Creates a string for to display the primary design units for the particular ip.
fn format_units_table(table: Vec<PrimaryUnit>) -> String {
    let header = format!("\
{:<32}{:<12}{:<9}
{:->32}{3:->12}{3:->9}\n",
                "Identifier", "Unit", "Public", " ");
    let mut body = String::new();

    let mut table = table;
    table.sort_by(|a, b| a.as_iden().unwrap().cmp(b.as_iden().unwrap()));
    for unit in table {
        body.push_str(&format!("{:<32}{:<12}{:<2}\n", 
            unit.as_iden().unwrap().to_string(), 
            unit.to_string(), 
            "y"));
    }

    header + &body
}

/// Tracks the dev version, installed versions, and available versions
type VersionTable = (Option<Version>, Vec<Version>, Vec<Version>);

/// Creates a string for a version table for the particular ip.
fn format_version_table(table: VersionTable) -> String {
    let header = format!("\
{:<15}{:<9}
{:->15}{2:->9}\n",
                "Version", "Status", " ");
    // create a hashset of all available versions to form a list
    let mut btmap = BTreeMap::<Version, (bool, bool, bool)>::new();
    // log what version the dev ip is at
    if let Some(v) = table.0 {
        btmap.insert(v, (true, false, false));
    }
    // log the installation versions
    for v in table.1 {
        match btmap.get_mut(&v) {
            Some(entry) => entry.1 = true,
            None => { btmap.insert(v, (false, true, false)); () },
        }
    }
    // log the available versions
    for v in table.2 {
        match btmap.get_mut(&v) {
            Some(entry) => entry.1 = true,
            None => { btmap.insert(v, (false, false, true)); () },
        } 
    }
    // create body text
    let mut body = String::new();
    for (ver, status) in btmap.iter().rev() {
        body.push_str(&format!("{:<15}{:<2}{:<2}{:<2}\n", 
            ver.to_string(),
            { if status.0 { "D" } else { "" } },
            { if status.1 { "I" } else { "" } },
            { if status.2 { "A" } else { "" } },
        ));
    }
    header + &body
}

const HELP: &str = "\
Access information about an ip

Usage:
    orbit probe [options] <ip>

Args:
    <ip>               the pkgid to request data about

Options:
    --tags                      display the list of possible versions
    --range <version:version>   narrow the displayed version list
    --ver, -v <version>         select a particular existing ip version
    --units                     display primary design units within an ip
    --changes                   view the changelog
    --readme                    view the readme

Use 'orbit help query' to learn more about the command.
";