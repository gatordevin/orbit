use std::collections::HashMap;
use crate::interface::errors::CliError;
use crate::interface::arg::*;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Token {
    UnattachedArgument(usize, String),
    AttachedArgument(usize, String),
    Flag(usize),
    Switch(usize, char),
    Ignore(usize, String),
    Terminator(usize),
}

impl<'c> Drop for Cli<'c> {
    fn drop(&mut self) {
        println!("dropping!");
    }
}

impl Token {
    fn take_str(self) -> String {
        match self {
            Self::UnattachedArgument(_, s) => s,
            Self::AttachedArgument(_, s) => s,
            Self::Ignore(_, s) => s,
            _ => panic!("cannot call take_str on token without string"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cli<'c> {
    tokens: Vec<Option<Token>>,
    opt_store: HashMap<String, Vec<usize>>,
    known_args: Vec<Arg<'c>>,
}

impl<'c> Cli<'c> {
    pub fn new() -> Self {
        Cli {
            tokens: Vec::new(),
            opt_store: HashMap::new(),
            known_args: Vec::new(),
        }
    }

    pub fn tokenize<T: Iterator<Item=String>>(args: T) -> Self {
        let mut tokens = Vec::<Option<Token>>::new();
        let mut store = HashMap::new();
        let mut terminated = false;
        let mut args = args.skip(1).enumerate();
        while let Some((i, mut arg)) = args.next() {
            // ignore all input after detecting the terminator
            if terminated == true {
                tokens.push(Some(Token::Ignore(i, arg)));
            // handle an option
            } else if arg.starts_with('-') {
                // try to separate from '=' sign
                let mut value: Option<String> = None;
                let mut option: Option<String> = None;
                {
                    if let Some((opt, val)) = arg.split_once('=') {
                        option = Some(opt.to_string());
                        value = Some(val.to_string());
                    }
                }
                // update arg to be the value split by '='
                if let Some(opt) = option {
                    arg = opt;
                }
                // handle long flag signal
                if arg.starts_with("--") {
                    arg.replace_range(0..=1, "");
                    // caught the terminator (purely "--")
                    if arg.is_empty() {
                        tokens.push(Some(Token::Terminator(i)));
                        terminated = true;
                    // caught a 'long option' flag
                    } else {
                        store.entry(arg).or_insert(Vec::new()).push(tokens.len());
                        tokens.push(Some(Token::Flag(i)));
                    }
                // handle short flag signal
                } else {
                    let mut arg = arg.chars().skip(1);
                    // split switches into individual components
                    while let Some(c) = arg.next() {
                        store.entry(c.to_string()).or_insert(Vec::new()).push(tokens.len());
                        tokens.push(Some(Token::Switch(i, c)));
                    }
                }
                // caught an argument directly attached to an option
                if let Some(val) = value {
                    tokens.push(Some(Token::AttachedArgument(i, val)));
                }
            // caught an argument
            } else {
                tokens.push(Some(Token::UnattachedArgument(i, arg)));
            }
        }
        Cli { 
            tokens: tokens,
            opt_store: store,
            known_args: vec![],
        }
    }

    /// Takes out the next UnattachedArg from the token stream.
    fn next_uarg(&mut self) -> Option<String> {
        if let Some(p) = self.tokens
            .iter_mut()
            .find(|s| {
                match s {
                    Some(Token::UnattachedArgument(_, _)) | Some(Token::Terminator(_)) => true,
                    _ => false,
                }
            }) {
                if let Some(Token::Terminator(_)) = p {
                    None
                } else {
                    Some(p.take().unwrap().take_str())
                }
        } else {
            None
        }
    }

    /// Serves the next `Positional` value in the token stream parsed as `T`.
    /// 
    /// Errors if parsing fails.
    pub fn check_positional<'a, T: FromStr>(&mut self, p: Positional<'c>) -> Result<Option<T>, CliError<'c>> 
    where <T as FromStr>::Err: std::error::Error {
        self.known_args.push(Arg::Positional(p));
        match self.next_uarg() {
            Some(s) => {
                match s.parse::<T>() {
                    Ok(r) => Ok(Some(r)),
                    Err(e) => Err(CliError::BadType(self.known_args.pop().unwrap(), e.to_string())),
                }
            },
            None => {
                Ok(None)
            }
        }
    }

    /// Forces the next `Positional to exist from token stream.
    /// 
    /// Errors if parsing fails or if no unattached argument is left in the token stream.
    pub fn require_positional<'a, T: FromStr>(&mut self, p: Positional<'c>) -> Result<T, CliError<'c>> 
    where <T as FromStr>::Err: std::error::Error {
        if let Some(value) = self.check_positional(p)? {
            Ok(value)
        } else {
            Err(CliError::MissingPositional(self.known_args.pop().unwrap(), "usage".to_string()))
        }
    }

    /// Queries for a value of `Optional`.
    /// 
    /// Errors if there are multiple values or if parsing fails.
    pub fn check_option<'a, T: FromStr>(&mut self, o: Optional<'c>) -> Result<Option<T>, CliError<'c>>
    where <T as FromStr>::Err: std::error::Error {
        // collect information on where the flag can be found
        let locs_flag = self.take_flag_locs(o.get_flag_ref().get_name_ref());
        let locs_switch = if let Some(c) = o.get_flag_ref().get_switch_ref() {
            self.take_switch_locs(c)
        } else {
            None
        };
        self.known_args.push(Arg::Optional(o));
        let locs = Self::combine_locations(locs_flag, locs_switch);
        // pull values from where the option flags were found (including switch)
        let mut values = self.pull_flag(locs, true);
        match values.len() {
            1 => {
                if let Some(s) = values.pop().unwrap() {
                    let result = s.parse::<T>();
                    match result {
                        Ok(r) => Ok(Some(r)),
                        Err(e) => Err(CliError::BadType(self.known_args.pop().unwrap(), e.to_string()))
                    }
                } else {
                    Err(CliError::ExpectingValue(self.known_args.pop().unwrap()))
                }
            },
            0 => Ok(None),
            _ => Err(CliError::DuplicateOptions(self.known_args.pop().unwrap())),
        }
    }

    fn combine_locations(lhs: Option<Vec<usize>>, rhs: Option<Vec<usize>>) -> Vec<usize> {
        if let Some(mut u_lhs) = lhs {
            if let Some(u_rhs) = rhs {
                u_lhs.extend(u_rhs);
            }
            u_lhs
        } else if let Some(u_rhs) = rhs {
            u_rhs
        } else {
            vec![]
        }
    }

    /// Queries if a flag was raised once and only once. 
    /// 
    /// Errors if the flag has an attached value or was raised multiple times.
    pub fn check_flag<'a>(&mut self, f: Flag<'c>) -> Result<bool, CliError<'c>> {
        // collect information on where the flag can be found
        let locs_flag = self.take_flag_locs(f.get_name_ref());
        let locs_switch = if let Some(c) = f.get_switch_ref() {
            self.take_switch_locs(c)
        } else {
            None
        };
        self.known_args.push(Arg::Flag(f));
        let locs = Self::combine_locations(locs_flag, locs_switch);
        let mut occurences = self.pull_flag(locs, false);
        // verify there are no values attached to this flag
        if let Some(val) = occurences.iter_mut().find(|p| p.is_some()) {
            return Err(CliError::UnexpectedValue(self.known_args.pop().unwrap(), val.take().unwrap()));
        } else {
            match occurences.len() {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(CliError::DuplicateOptions(self.known_args.pop().unwrap())),
            }
        }
    }

    /// Accept a command's list of options before processing
    // fn learn_options(&mut self, &Vec<Arg>) {
    //     todo!()
    // }

    /// Find the first unattached argument that matches a possible subcommand name
    // fn detect_subcommand(&mut self, Vec<String>) {
    //     todo!()
    // }

    /// Grabs the flag/switch from the token stream, and collects. If an argument were to follow
    /// it will be in the vector.
    fn pull_flag(&mut self, m: Vec<usize>, with_uarg: bool) -> Vec<Option<String>> {
        // remove flags
        m.iter().map(|f| {
            // remove the flag instance from the token stream
            self.tokens.get_mut(*f).unwrap().take();
            // check the next position for a value
            if let Some(t_next) = self.tokens.get_mut(*f+1) {
                match t_next {
                    Some(Token::AttachedArgument(_, _)) => {
                        Some(t_next.take().unwrap().take_str())
                    }
                    Some(Token::UnattachedArgument(_, _)) => {
                        // do not take unattached arguments unless told by parameter
                        if with_uarg == false {
                            None
                        } else {
                            Some(t_next.take().unwrap().take_str())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }).collect()
    }

    /// Removes the ignored tokens from the stream, if they exist.
    fn get_remainder(&mut self) -> Vec<String> {
        self.tokens.iter_mut().skip_while(|p| {
            match p {
                Some(Token::Terminator(_)) => false,
                _ => true,
            }
        }).filter_map(|f| {
            match f {
                Some(Token::Ignore(_, _)) => {
                    Some(f.take().unwrap().take_str())
                }
                _ => None
            }
        }).collect()
    }

    /// Returns all locations in the token stream where the flag is found.
    fn take_flag_locs(&mut self, s: &str) -> Option<Vec<usize>> {
        Some(self.opt_store.remove(s)?)
    }

    /// Returns all locations in the token stream where the switch is found.
    fn take_switch_locs(&mut self, c: &char) -> Option<Vec<usize>> {
        // allocate &str to the stack and not the heap to get from store
        let mut tmp = [0; 4];
        let m = c.encode_utf8(&mut tmp);
        Some(self.opt_store.remove(m)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Helper test fn to write vec of &str as iterator for Cli parameter.
    fn args<'a>(args: Vec<&'a str>) -> Box<dyn Iterator<Item=String> + 'a> {
        Box::new(args.into_iter().map(|f| f.to_string()).into_iter())
    }

    #[test]
    fn tokenizer() {
        let cli = Cli::tokenize(args(vec![]));
        assert_eq!(cli.tokens, vec![]);

        let cli = Cli::tokenize(args(vec!["orbit"]));
        assert_eq!(cli.tokens, vec![]);

        let cli = Cli::tokenize(args(vec!["orbit", "--help"]));
        assert_eq!(cli.tokens, vec!
            [Some(Token::Flag(0))
            ]
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-v"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0)), 
            Some(Token::Switch(1, 'v'))
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "new", "rary.gates"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::UnattachedArgument(0, "new".to_string())), 
            Some(Token::UnattachedArgument(1, "rary.gates".to_string())),
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-vh"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0)), 
            Some(Token::Switch(1, 'v')),
            Some(Token::Switch(1, 'h')),
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-vhc=10"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0)), 
            Some(Token::Switch(1, 'v')),
            Some(Token::Switch(1, 'h')),
            Some(Token::Switch(1, 'c')),
            Some(Token::AttachedArgument(1, "10".to_string())),
            ],
        );

        // an attached argument can sneak in behind a terminator
        let cli = Cli::tokenize(args(vec!["orbit", "--=value", "extra"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Terminator(0)),
            Some(Token::AttachedArgument(0, "value".to_string())),
            Some(Token::Ignore(1, "extra".to_string())),
        ]);

        // final boss
        let cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-sci", "--", "--map", "synthesis", "-jto"]
        ));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0)),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4)),
            Some(Token::Flag(5)),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(6)),
            Some(Token::Switch(7, 's')),
            Some(Token::Switch(7, 'c')),
            Some(Token::Switch(7, 'i')),
            Some(Token::Terminator(8)),
            Some(Token::Ignore(9, "--map".to_string())),
            Some(Token::Ignore(10, "synthesis".to_string())),
            Some(Token::Ignore(11, "-jto".to_string())),
            ],
        );
    }

    #[test]
    fn find_flags_and_switches() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-sci", "-i", "--", "--map", "synthesis", "-jto"]
        ));

        // detects 0
        assert_eq!(cli.take_flag_locs("version"), None);
        // detects 1
        assert_eq!(cli.take_flag_locs("lib"), Some(vec![4]));
        // detects multiple
        assert_eq!(cli.take_flag_locs("help"), Some(vec![0, 7]));
        // flag was past terminator and marked as ignore
        assert_eq!(cli.take_flag_locs("map"), None);
        // filters out arguments
        assert_eq!(cli.take_flag_locs("rary.gates"), None);

        // detects 0
        assert_eq!(cli.take_switch_locs(&'q'), None);
        // detects 1
        assert_eq!(cli.take_switch_locs(&'v'), Some(vec![1]));
        // detects multiple
        assert_eq!(cli.take_switch_locs(&'i'), Some(vec![10, 11]));
        // switch was past terminator and marked as ignore
        assert_eq!(cli.take_switch_locs(&'j'), None);
    }

    #[test]
    fn flags_in_map() {
        let cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-sci", "--", "--map", "synthesis", "-jto"]
        ));
        let mut opt_store = HashMap::<String, Vec<usize>>::new();
        // store long options
        opt_store.insert("help".to_string(), vec![0, 7]);
        opt_store.insert("lib".to_string(), vec![4]);
        opt_store.insert("name".to_string(), vec![5]);
        // stores switches too
        opt_store.insert("v".to_string(), vec![1]);
        opt_store.insert("s".to_string(), vec![8]);
        opt_store.insert("c".to_string(), vec![9]);
        opt_store.insert("i".to_string(), vec![10]);
        assert_eq!(cli.opt_store, opt_store);
    }

    #[test]
    fn take_unattached_args() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-scii", "get", "--", "--map", "synthesis", "-jto"]
        ));

        assert_eq!(cli.next_uarg().unwrap(), "new".to_string());
        assert_eq!(cli.next_uarg().unwrap(), "ip".to_string());
        assert_eq!(cli.next_uarg().unwrap(), "get".to_string());
        assert_eq!(cli.next_uarg(), None);
    }

    #[test]
    fn take_remainder_args() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-scii", "get", "--", "--map", "synthesis", "-jto"]
        ));
        assert_eq!(cli.get_remainder(), vec!["--map", "synthesis", "-jto"]);
        // the items were removed from the token stream
        assert_eq!(cli.get_remainder(), Vec::<String>::new());

        // an attached argument can sneak in behind a terminator (handle in a result fn)
        let mut cli = Cli::tokenize(args(vec!["orbit", "--=value", "extra"]));
        assert_eq!(cli.get_remainder(), vec!["extra"]);
    }

    #[test]
    fn pull_values_from_flags() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help"],
        ));
        let locs = cli.take_flag_locs("help").unwrap();
        assert_eq!(cli.pull_flag(locs, false), vec![None]);
        assert_eq!(cli.tokens.get(0), Some(&None));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--name", "gates", "arg", "--lib", "new", "--name=gates2", "--opt=1", "--opt", "--help"]
        ));
        let locs = cli.take_flag_locs("lib").unwrap();
        assert_eq!(cli.pull_flag(locs, false), vec![None]);
        // token no longer exists
        assert_eq!(cli.tokens.get(3), Some(&None));

        // gets strings and removes both instances of flag from token stream
        let locs = cli.take_flag_locs("name").unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![Some("gates".to_string()), Some("gates2".to_string())]);
        assert_eq!(cli.tokens.get(0), Some(&None));
        assert_eq!(cli.tokens.get(5), Some(&None));

        let locs = cli.take_flag_locs("opt").unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![Some("1".to_string()), None]);

        // gets switches as well from the store
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--name", "gates", "-sicn", "dut", "new", "-vl=direct", "--help", "-l", "-m", "install"]
        ));
        let locs = cli.take_switch_locs(&'l').unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![Some("direct".to_string()), None]);
        assert_eq!(cli.tokens.get(9), Some(&None));
        assert_eq!(cli.tokens.get(12), Some(&None));
        let locs = cli.take_switch_locs(&'s').unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![None]);
        let locs = cli.take_switch_locs(&'v').unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![None]);
        let locs = cli.take_switch_locs(&'i').unwrap();
        assert_eq!(cli.pull_flag(locs, true), vec![None]);
        let locs = cli.take_switch_locs(&'c').unwrap();
        assert_eq!(cli.pull_flag(locs, false), vec![None]);
        let locs = cli.take_switch_locs(&'m').unwrap();
        assert_eq!(cli.pull_flag(locs, false), vec![None]);
    }

    #[test]
    fn check_flag() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "--verbose", "get"]
        ));
        assert_eq!(cli.check_flag(Flag::new("help")), Ok(true));
        assert_eq!(cli.check_flag(Flag::new("verbose")), Ok(true));
        assert_eq!(cli.check_flag(Flag::new("version")), Ok(false));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-h"]
        ));
        assert_eq!(cli.check_flag(Flag::new("help").switch('h')), Err(CliError::DuplicateOptions(Arg::Flag(Flag::new("help").switch('h')))));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--help", "--help", "--version=9"]
        ));
        assert_eq!(cli.check_flag(Flag::new("help")), Err(CliError::DuplicateOptions(Arg::Flag(Flag::new("help")))));
        assert_eq!(cli.check_flag(Flag::new("version")), Err(CliError::UnexpectedValue(Arg::Flag(Flag::new("version")), "9".to_string())));
    }

    #[test]
    fn check_positional() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "new", "rary.gates"]
        ));
        assert_eq!(cli.check_positional::<String>(Positional::new("command")), Ok(Some("new".to_string())));
        assert_eq!(cli.check_positional::<String>(Positional::new("ip")), Ok(Some("rary.gates".to_string())));
        assert_eq!(cli.check_positional::<i32>(Positional::new("path")), Ok(None));
    }

    #[test]
    fn check_option() {
        let mut cli = Cli::tokenize(args(
            vec!["orbit", "command", "--rate", "10"]
        ));
        assert_eq!(cli.check_option(Optional::new("rate")), Ok(Some(10)));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--flag", "--rate=9", "command", "-r", "14"]
        ));
        assert_eq!(cli.check_option::<i32>(Optional::new("rate").switch('r')), Err(CliError::DuplicateOptions(Arg::Optional(Optional::new("rate").switch('r')))));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--flag", "-r", "14"]
        ));
        assert_eq!(cli.check_option(Optional::new("rate").switch('r')), Ok(Some(14)));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--flag", "--rate", "--verbose"]
        ));
        assert_eq!(cli.check_option::<i32>(Optional::new("rate")), Err(CliError::ExpectingValue(Arg::Optional(Optional::new("rate")))));

        let mut cli = Cli::tokenize(args(
            vec!["orbit", "--flag", "--rate", "five", "--verbose"]
        ));
        assert!(cli.check_option::<i32>(Optional::new("rate")).is_err());
    }

    #[test]
    fn take_token_str() {
        let t = Token::UnattachedArgument(0, "get".to_string());
        // consumes token and returns its internal string
        assert_eq!(t.take_str(), "get");

        let t = Token::AttachedArgument(1, "rary.gates".to_string());
        assert_eq!(t.take_str(), "rary.gates");

        let t = Token::Ignore(7, "--map".to_string());
        assert_eq!(t.take_str(), "--map");
    }

    #[test]
    #[should_panic]
    fn take_impossible_token_flag_str() {
        let t = Token::Flag(7);
        t.take_str();
    }

    #[test]
    #[should_panic]
    fn take_impossible_token_switch_str() {
        let t = Token::Switch(7, 'h');
        t.take_str();
    }

    #[test]
    #[should_panic]
    fn take_impossible_token_terminator_str() {
        let t = Token::Terminator(9);
        t.take_str();
    }
}

// orbit --help -v new ip --lib --name=rary.gates -sci -- --map synthesis

// noop Flag Switch Arg Arg Flag Flag Arg Switch Switch Switch Flag Ignore Ignore

// grammar, collect tokens
/*

Subcommand -> 


*/

/*
Build a command (ideal)

struct DoSomething {
    action: String,
    repeat: Option<u8>,
}


fn new() -> Self {
    DoSomething {
        action: args.match(Positional::new("action"))
    }
}


*/