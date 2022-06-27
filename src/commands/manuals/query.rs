// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    query - probe information about an orbit IP

SYNOPSIS
    orbit query [options] <ip>

DESCRIPTION
    This command will print information to the console for the user to learn
    more information about any given IP. By default, it will return the
    Orbit.toml manifest file contents.
      
    Since IP can exist at 3 different levels, the default IP manifest to return
    data about is the latest installed version. If there is none, it will try
    the latest available version, and then the development version, if those 
    exist. 
      
    The --version option can accept a partial or full verion value, the word
    'latest', or 'dev'. Latest will point to the user's highest known version,
    while 'dev' will point to the IP in the DEV_PATH.

OPTIONS
    <ip>  
          The fully specified pkgid for the ip
     
    --tags  
          Return a list of versions and where they are located
     
    --ver, -v <version>  
          Extract data from a particular version
     
    --units  
          List the available primary design units within the IP

EXAMPLES
    orbit query ks-tech.rary.gates --tags
    orbit query util.toolbox -v 1.2.3 --units
";