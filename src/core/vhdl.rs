//! VHDL tokenizer

#[derive(Debug, PartialEq, Clone)]
/// (Line, Col)
struct Position(usize, usize);

impl Position {
    /// Creates a new `Position` struct as line 1, col 0.
    fn new() -> Self {
        Position(1, 0)
    }

    /// Increments the column counter by 1.
    fn next_col(&mut self) {
        self.1 += 1;
    }   

    /// Increments the column counter by 1. If the current char `c` is a newline,
    /// it will then drop down to the next line.
    fn step(&mut self, c: &char) {
        if c == &'\n' {
            self.next_line();
        }
        // @TODO step by +4 if encountered a tab?
        self.next_col();
    }

    /// Increments the line counter by 1.
    /// 
    /// Also resets the column counter to 0.
    fn next_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Access the line (`.0`) number.
    fn line(&self) -> usize {
        self.0
    }

    /// Access the col (`.1`) number.
    fn col(&self) -> usize {
        self.1
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
struct Token<T> {
    position: Position,
    ttype: T,
}

impl<T> Token<T> {
    /// Reveals the token type.
    fn unwrap(&self) -> &T {
        &self.ttype
    }

    /// Transforms the token into its type.
    fn take(self) -> T {
        self.ttype
    }

    /// Returns the position in the file where the token was captured.
    fn locate(&self) -> &Position {
        &self.position
    }

    /// Creates a new token.
    fn new(ttype: T, loc: Position) -> Self {
        Self {
            position: loc,
            ttype: ttype,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Comment {
    Single(String),
    Delimited(String),
}

impl Comment {
    fn as_str(&self) -> &str {
        match self {
            Self::Single(note) => note.as_ref(),
            Self::Delimited(note) => note.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Character(String);

impl Character {
    fn new(c: char) -> Self {
        Self(String::from(c))
    }

    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

#[derive(Debug, PartialEq)]
struct BitStrLiteral {
    width: Option<usize>,
    base: BaseSpec,
    literal: String,
}

impl BitStrLiteral {
    fn as_str(&self) -> &str {
        &self.literal
    }

    fn new(b: BaseSpec) -> Self {
        Self {
            width: None,
            base: b,
            literal: String::new(),
        }
    }

    fn literal(mut self, s: String) -> Self {
        self.literal = s;
        self
    }

    fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }
}

// B|O|X|UB|UO|UX|SB|SO|SX|D
#[derive(Debug, PartialEq)]
enum BaseSpec {
    B,
    O,
    X,
    UB,
    UO,
    UX,
    SB,
    SO,
    SX,
    D
}

impl std::str::FromStr for BaseSpec {
    type Err = (); // @TODO handle errors
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "b"  => Self::B,
            "o"  => Self::O,
            "x"  => Self::X,
            "ub" => Self::UB,
            "uo" => Self::UO,
            "ux" => Self::UX,
            "sb" => Self::SB,
            "so" => Self::SO,
            "sx" => Self::SX,
            "d"  => Self::D,
            _ => panic!("invalid base specifier {}", s)
        })
    }
}

impl BaseSpec {
    fn as_str(&self) -> &str {
        match self {
            Self::B => "b",
            Self::O => "o",
            Self::X => "x",
            Self::UB => "ub",
            Self::UO => "uo",
            Self::UX => "ux",
            Self::SB => "sb",
            Self::SO => "so",
            Self::SX => "sx",
            Self::D => "d",
        }
    }
}

#[derive(Debug)]
enum Identifier {
    Basic(String),
    Extended(String),
}

impl Identifier {
    // Returns the reference to the inner `String` struct.
    fn as_str(&self) -> &str {
        match self {
            Self::Basic(id) => id.as_ref(),
            Self::Extended(id) => id.as_ref(),
        }
    }

    /// Checks if `self` is an extended identifier or not.
    fn is_extended(&self) -> bool {
        match self {
            Self::Extended(_) => true,
            Self::Basic(_) => false,
        }
    }
}

impl std::cmp::Eq for Identifier {}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // instantly not equal if not they are not of same type
        if self.is_extended() != other.is_extended() { return false };
        // compare with case sensitivity
        if self.is_extended() == true {
            self.as_str() == other.as_str()
        // compare without case sensitivity
        } else {
            cmp_ignore_case(self.as_str(), other.as_str())
        }
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Extended(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AbstLiteral {
    Decimal(String),
    Based(String),
}

impl AbstLiteral {
    fn as_str(&self) -> &str {
        match self {
            Self::Decimal(val) => val.as_ref(),
            Self::Based(val) => val.as_ref(),
        }
    }
}

trait Tokenize {
    type TokenType;
    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>>;
} 

#[derive(Debug, PartialEq)]
enum VHDLToken {
    Comment(Comment),               // (String) 
    Identifier(Identifier),         // (String) ...can be general or extended (case-sensitive) identifier
    AbstLiteral(AbstLiteral),       // (String)
    CharLiteral(Character),         // (char)
    StrLiteral(String),             // (String)
    BitStrLiteral(BitStrLiteral),   // (String)
    EOF,
    // --- delimiters
    Ampersand,      // &
    SingleQuote,    // '
    ParenL,         // (
    ParenR,         // )
    Star,           // *
    Plus,           // +
    Comma,          // ,
    Dash,           // -
    Dot,            // .
    FwdSlash,       // /
    Colon,          // :
    Terminator,     // ;
    Lt,             // <
    Eq,             // =
    Gt,             // >
    BackTick,       // `
    Pipe,           // | or ! VHDL-1993 LRM p180
    BrackL,         // [
    BrackR,         // ]
    Question,       // ?
    AtSymbol,       // @
    Arrow,          // =>
    DoubleStar,     // **
    VarAssign,      // :=
    Inequality,     // /=
    GTE,            // >=
    SigAssign,      // <=
    Box,            // <>
    SigAssoc,       // <=>
    CondConv,       // ??
    MatchEQ,        // ?=
    MatchNE,        // ?/=
    MatchLT,        // ?<
    MatchLTE,       // ?<=
    MatchGT,        // ?>
    MatchGTE,       // ?>=
    DoubleLT,       // <<
    DoubleGT,       // >>
    // --- keywords
    Abs,
    Access,
    After,
    Alias,
    All,
    And,
    Architecture,
    Array,
    Assert,
    Assume,
    // AssumeGuarantee is omitted from VHDL-2019 LRM
    Attribute,
    Begin,
    Block,
    Body,
    Buffer,
    Bus,
    Case, 
    Component,
    Configuration,
    Constant, 
    Context,
    Cover,
    Default,
    Disconnect, 
    Downto,
    Else, 
    Elsif,
    End,
    Entity, 
    Exit,
    Fairness,
    File,
    For, 
    Force,
    Function,
    Generate, 
    Generic, 
    Group, 
    Guarded,
    If,
    Impure, 
    In, 
    Inertial, 
    Inout, 
    Is,
    Label, 
    Library, 
    Linkage, 
    Literal, 
    Loop,
    Map,
    Mod,
    Nand,
    New, 
    Next, 
    Nor, 
    Not, 
    Null,
    Of,
    On,
    Open,
    Or, 
    Others, 
    Out,
    Package, 
    Parameter, 
    Port, 
    Postponed, 
    Private,
    Procedure, 
    Process, 
    Property, 
    Protected, 
    Pure,
    Range,
    Record,
    Register,
    Reject,
    Release,
    Rem,
    Report,
    Restrict, 
    // RestrictGuarantee is omitted from VHDL-2019 LRM
    Return,
    Rol, 
    Ror,
    Select, 
    Sequence, 
    Severity,
    Signal, 
    Shared, 
    Sla,
    Sll,
    Sra,
    Srl, 
    Strong, 
    Subtype,
    Then,
    To, 
    Transport, 
    Type,
    Unaffected, 
    Units,
    Until,
    Use,
    Variable, 
    View,
    Vmode, 
    Vpkg,
    Vprop, 
    Vunit,
    Wait, 
    When, 
    While, 
    With,
    Xnor, 
    Xor,
}

/// Walks through the possible interpretations for capturing a VHDL delimiter.
/// 
/// If it successfully finds a valid VHDL delimiter, it will move the `loc` the number
/// of characters it consumed.
fn collect_delimiter<T>(stream: &mut Peekable<T>, loc: &mut Position, c0: Option<char>) -> Option<VHDLToken> 
    where T: Iterator<Item=char> {

    let mut delim = String::with_capacity(3);
    if let Some(c) = c0 {
        delim.push(c);
    }

    while let Some(c) = stream.peek() {
        match delim.len() {
            0 => match c {
                // ambiguous characters...read another character (could be a len-2 delimiter)
                '?' | '<' | '>' | '/' | '=' | '*' | ':' => {
                    loc.next_col();
                    delim.push(stream.next().unwrap())
                },
                _ => { 
                    let op = VHDLToken::match_delimiter(&String::from(c.clone()));
                    // if it was a delimiter, take the character and increment the location
                    if let Some(r) = op {
                        loc.next_col();
                        stream.next();
                        return Some(r)
                    } else {
                        return None
                    }
                }
            }
            1 => match delim.chars().nth(0).unwrap() {
                '?' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '/' | '<' | '>' => {
                            loc.next_col();
                            delim.push(stream.next().unwrap())
                        }
                        _ => { return Some(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                '<' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '=' => {
                            loc.next_col();
                            delim.push(stream.next().unwrap())
                        },
                        _ => { return Some(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                _ => {
                    // try with 2
                    delim.push(c.clone());
                    if let Some(op) = VHDLToken::match_delimiter(&delim) {
                        loc.next_col();
                        stream.next();
                        return Some(op)
                    } else {
                        // revert back to 1
                        delim.pop();
                        return VHDLToken::match_delimiter(&delim)
                    }
                }
            }
            2 => {
                // try with 3
                delim.push(c.clone());
                if let Some(op) = VHDLToken::match_delimiter(&delim) {
                    stream.next();
                    loc.next_col();
                    return Some(op)
                } else {
                    // revert back to 2 (guaranteed to exist)
                    delim.pop();
                    return Some(VHDLToken::match_delimiter(&delim).expect("invalid token"))
                }
            }
            _ => panic!("delimiter matching exceeds 3 characters")
        }
    };
    // try when hiting end of stream
    VHDLToken::match_delimiter(&delim)
}

impl VHDLToken {
    /// Attempts to match the given string of characters `s` to a VHDL delimiter.
    fn match_delimiter(s: &str) -> Option<Self> {
        Some(match s {
            "&"     => Self::Ampersand,    
            "'"     => Self::SingleQuote,  
            "("     => Self::ParenL,       
            ")"     => Self::ParenR,       
            "*"     => Self::Star,         
            "+"     => Self::Plus,         
            ","     => Self::Comma,        
            "-"     => Self::Dash,         
            "."     => Self::Dot,          
            "/"     => Self::FwdSlash,     
            ":"     => Self::Colon,        
            ";"     => Self::Terminator,   
            "<"     => Self::Lt,           
            "="     => Self::Eq,           
            ">"     => Self::Gt,           
            "`"     => Self::BackTick,     
      "!" | "|"     => Self::Pipe,         
            "["     => Self::BrackL,       
            "]"     => Self::BrackR,       
            "?"     => Self::Question,     
            "@"     => Self::AtSymbol,     
            "=>"    => Self::Arrow,          
            "**"    => Self::DoubleStar,     
            ":="    => Self::VarAssign,      
            "/="    => Self::Inequality,     
            ">="    => Self::GTE,            
            "<="    => Self::SigAssign,      
            "<>"    => Self::Box,            
            "<=>"   => Self::SigAssoc,       
            "??"    => Self::CondConv,       
            "?="    => Self::MatchEQ,        
            "?/="   => Self::MatchNE,      
            "?<"    => Self::MatchLT,        
            "?<="   => Self::MatchLTE,       
            "?>"    => Self::MatchGT,        
            "?>="   => Self::MatchGTE,       
            "<<"    => Self::DoubleLT,       
            ">>"    => Self::DoubleGT,       
            _ => return None,
        })
    }

    /// Attempts to match the given string of characters `s` to a VHDL keyword.
    /// 
    /// Compares `s` against keywords using ascii lowercase comparison.
    fn match_keyword(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_ref() {
            "abs"           => Self::Abs, 
            "access"        => Self::Access, 
            "after"         => Self::After, 
            "alias"         => Self::Alias, 
            "all"           => Self::All, 
            "and"           => Self::And, 
            "architecture"  => Self::Architecture, 
            "array"         => Self::Array, 
            "assert"        => Self::Assert, 
            "assume"        => Self::Assume, 
            "attribute"     => Self::Attribute, 
            "begin"         => Self::Begin, 
            "block"         => Self::Block, 
            "body"          => Self::Body, 
            "buffer"        => Self::Buffer, 
            "bus"           => Self::Bus, 
            "case"          => Self::Case, 
            "component"     => Self::Component, 
            "configuration" => Self::Configuration, 
            "constant"      => Self::Constant, 
            "context"       => Self::Context, 
            "cover"         => Self::Cover, 
            "default"       => Self::Default, 
            "disconnect"    => Self::Disconnect, 
            "downto"        => Self::Downto, 
            "else"          => Self::Else, 
            "elsif"         => Self::Elsif, 
            "end"           => Self::End, 
            "entity"        => Self::Entity, 
            "exit"          => Self::Exit, 
            "fairness"      => Self::Fairness, 
            "file"          => Self::File, 
            "for"           => Self::For, 
            "force"         => Self::Force, 
            "function"      => Self::Function, 
            "generate"      => Self::Generate, 
            "generic"       => Self::Generic, 
            "group"         => Self::Group, 
            "guarded"       => Self::Guarded, 
            "if"            => Self::If, 
            "impure"        => Self::Impure, 
            "in"            => Self::In, 
            "inertial"      => Self::Inertial, 
            "inout"         => Self::Inout, 
            "is"            => Self::Is, 
            "label"         => Self::Label, 
            "library"       => Self::Library, 
            "linkage"       => Self::Linkage, 
            "literal"       => Self::Literal, 
            "loop"          => Self::Loop, 
            "map"           => Self::Map, 
            "mod"           => Self::Mod, 
            "nand"          => Self::Nand, 
            "new"           => Self::New, 
            "next"          => Self::Next, 
            "nor"           => Self::Nor, 
            "not"           => Self::Not, 
            "null"          => Self::Null, 
            "of"            => Self::Of, 
            "on"            => Self::On, 
            "open"          => Self::Open, 
            "or"            => Self::Or, 
            "others"        => Self::Others, 
            "out"           => Self::Out, 
            "package"       => Self::Package, 
            "parameter"     => Self::Parameter, 
            "port"          => Self::Port, 
            "postponed"     => Self::Postponed, 
            "private"       => Self::Private, 
            "procedure"     => Self::Procedure, 
            "process"       => Self::Process, 
            "property"      => Self::Property, 
            "protected"     => Self::Protected, 
            "pure"          => Self::Pure, 
            "range"         => Self::Range, 
            "record"        => Self::Record, 
            "register"      => Self::Register, 
            "reject"        => Self::Reject, 
            "release"       => Self::Release, 
            "rem"           => Self::Rem, 
            "report"        => Self::Report, 
            "restrict"      => Self::Restrict, 
            "return"        => Self::Return, 
            "rol"           => Self::Rol, 
            "ror"           => Self::Ror, 
            "select"        => Self::Select, 
            "sequence"      => Self::Sequence, 
            "severity"      => Self::Severity, 
            "signal"        => Self::Signal, 
            "shared"        => Self::Shared, 
            "sla"           => Self::Sla, 
            "sll"           => Self::Sll, 
            "sra"           => Self::Sra, 
            "srl"           => Self::Srl, 
            "strong"        => Self::Strong, 
            "subtype"       => Self::Subtype, 
            "then"          => Self::Then, 
            "to"            => Self::To, 
            "transport"     => Self::Transport, 
            "type"          => Self::Type, 
            "unaffected"    => Self::Unaffected, 
            "units"         => Self::Units, 
            "until"         => Self::Until, 
            "use"           => Self::Use, 
            "variable"      => Self::Variable, 
            "view"          => Self::View, 
            "vmode"         => Self::Vmode, 
            "vpkg"          => Self::Vpkg, 
            "vprop"         => Self::Vprop, 
            "vunit"         => Self::Vunit, 
            "wait"          => Self::Wait, 
            "when"          => Self::When, 
            "while"         => Self::While, 
            "with"          => Self::With, 
            "xnor"          => Self::Xnor, 
            "xor"           => Self::Xor, 
            _ => return None
        })
    }
}

impl std::fmt::Display for VHDLToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Comment(note) => note.as_str(),
            Self::Identifier(id) => id.as_str(),
            Self::AbstLiteral(a) => a.as_str(),
            Self::CharLiteral(c) => c.as_str(),
            Self::StrLiteral(s) => s.as_ref(),
            Self::BitStrLiteral(b) => b.as_str(),
            Self::EOF           => "EOF",
            // --- delimiters
            Self::Ampersand     => "&",
            Self::SingleQuote   => "'",
            Self::ParenL        => "(",
            Self::ParenR        => ")",
            Self::Star          => "*",
            Self::Plus          => "+",
            Self::Comma         => ",",
            Self::Dash          => "-",
            Self::Dot           => ".",
            Self::FwdSlash      => "/",
            Self::Colon         => ":",
            Self::Terminator    => ";",
            Self::Lt            => "<",
            Self::Eq            => "=",
            Self::Gt            => ">",
            Self::BackTick      => "`",
            Self::Pipe          => "|",
            Self::BrackL        => "[",
            Self::BrackR        => "]",
            Self::Question      => "?",
            Self::AtSymbol      => "@",
            Self::Arrow         => "=>",
            Self::DoubleStar    => "**",
            Self::VarAssign     => ":=",
            Self::Inequality    => "/=",
            Self::GTE           => ">=",
            Self::SigAssign     => "<=",
            Self::Box           => "<>",
            Self::SigAssoc      => "<=>",
            Self::CondConv      => "??",
            Self::MatchEQ       => "?=",
            Self::MatchNE       => "?/=",
            Self::MatchLT       => "?<",
            Self::MatchLTE      => "?<=",
            Self::MatchGT       => "?>",
            Self::MatchGTE      => "?>=",
            Self::DoubleLT      => "<<",
            Self::DoubleGT      => ">>",
            // --- keywords
            Self::Abs           => "abs",
            Self::Access        => "access",
            Self::After         => "after",
            Self::Alias         => "alias",
            Self::All           => "all",
            Self::And           => "and", 
            Self::Architecture  => "architecture",
            Self::Array         => "array",
            Self::Assert        => "assert",
            Self::Assume        => "assume",
            Self::Attribute     => "attribute",
            Self::Begin         => "begin",
            Self::Block         => "block",
            Self::Body          => "body",
            Self::Buffer        => "buffer",
            Self::Bus           => "bus",
            Self::Case          => "case", 
            Self::Component     => "component",
            Self::Configuration => "configuration",
            Self::Constant      => "constant", 
            Self::Context       => "context",
            Self::Cover         => "cover",
            Self::Default       => "default",
            Self::Disconnect    => "disconnect", 
            Self::Downto        => "downto",
            Self::Else          => "else", 
            Self::Elsif         => "elsif",
            Self::End           => "end",
            Self::Entity        => "entity", 
            Self::Exit          => "exit",
            Self::Fairness      => "fairness",
            Self::File          => "file",
            Self::For           => "for", 
            Self::Force         => "force",
            Self::Function      => "function",
            Self::Generate      => "generate", 
            Self::Generic       => "generic", 
            Self::Group         => "group", 
            Self::Guarded       => "guarded",
            Self::If            => "if",
            Self::Impure        => "impure", 
            Self::In            => "in",     
            Self::Inertial      => "inertial", 
            Self::Inout         => "inout", 
            Self::Is            => "is",
            Self::Label         => "label", 
            Self::Library       => "library", 
            Self::Linkage       => "linkage", 
            Self::Literal       => "literal", 
            Self::Loop          => "loop",
            Self::Map           => "map",
            Self::Mod           => "mod",
            Self::Nand          => "nand",
            Self::New           => "new", 
            Self::Next          => "next", 
            Self::Nor           => "nor", 
            Self::Not           => "not", 
            Self::Null          => "null",
            Self::Of            => "of",
            Self::On            => "on",
            Self::Open          => "open",
            Self::Or            => "or", 
            Self::Others        => "others", 
            Self::Out           => "out",
            Self::Package       => "package", 
            Self::Parameter     => "parameter", 
            Self::Port          => "port", 
            Self::Postponed     => "postponed", 
            Self::Private       => "private",
            Self::Procedure     => "procedure", 
            Self::Process       => "process", 
            Self::Property      => "property", 
            Self::Protected     => "protected", 
            Self::Pure          => "pure",
            Self::Range         => "range",
            Self::Record        => "record",    
            Self::Register      => "register",
            Self::Reject        => "reject",
            Self::Release       => "release",
            Self::Rem           => "rem",
            Self::Report        => "report",
            Self::Restrict      => "restrict", 
            Self::Return        => "return",
            Self::Rol           => "rol", 
            Self::Ror           => "ror",
            Self::Select        => "select", 
            Self::Sequence      => "sequence", 
            Self::Severity      => "severity",
            Self::Signal        => "signal", 
            Self::Shared        => "shared", 
            Self::Sla           => "sla",
            Self::Sll           => "sll",
            Self::Sra           => "sra",
            Self::Srl           => "srl", 
            Self::Strong        => "strong", 
            Self::Subtype       => "subtype",
            Self::Then          => "then",
            Self::To            => "to", 
            Self::Transport     => "transport", 
            Self::Type          => "type",
            Self::Unaffected    => "unaffected", 
            Self::Units         => "units",
            Self::Until         => "until",
            Self::Use           => "use",
            Self::Variable      => "variable", 
            Self::View          => "view",
            Self::Vmode         => "vmode", 
            Self::Vpkg          => "vpkg",
            Self::Vprop         => "vprop", 
            Self::Vunit         => "vunit",
            Self::Wait          => "wait", 
            Self::When          => "when", 
            Self::While         => "while", 
            Self::With          => "with",
            Self::Xnor          => "xnor", 
            Self::Xor           => "xor",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq)]
struct VHDLTokenizer {
    inner: Vec<Token<VHDLToken>>,
}

impl std::fmt::Debug for VHDLTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.inner {
            write!(f, "{} {}\n", tk.locate(), tk.unwrap())?
        }
        Ok(())
    } 
}

/// Compares to string references `s0` and `s1` with case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding case sensivity.
fn cmp_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_lowercase().cmp(s1.next().unwrap().to_lowercase()) != std::cmp::Ordering::Equal {
            return false
        }
    }
    true
}

/// Compares to string references `s0` and `s1` with only ascii case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding ascii case sensivity.
fn cmp_ascii_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_ascii_lowercase() != s1.next().unwrap().to_ascii_lowercase() {
            return false
        }
    }
    true
}

use std::iter::Peekable;

/// Walks through the stream to gather a `String` literal until finding the 
/// exiting character `br`.
/// 
/// An escape is allowed by double placing the `br`, i.e. """hello"" world".
/// Assumes the first token to parse in the stream is not the `br` character.
/// The `loc` stays up to date on its position in the file.
fn enclose<T>(br: &char, stream: &mut Peekable<T>, loc: &mut Position) -> String 
    where T: Iterator<Item=char> {
        let mut result = String::new();
        while let Some(c) = stream.next() {
            loc.next_col();
            // verify it is a graphic character
            if char_set::is_graphic(&c) == false { panic!("invalid character {}", c) }
            // detect escape sequence
            if br == &c {
                match stream.peek() {
                    Some(c_next) => if br == c_next {
                        loc.next_col();
                        stream.next(); // skip over escape character
                    } else {
                        break;
                    }
                    None => break,
                }
            } 
            result.push(c);
        }
        result
}

mod char_set {
    pub const ASCII_ZERO: usize = '0' as usize;
    pub const DOUBLE_QUOTE: char = '\"';
    pub const BACKSLASH: char = '\\';
    pub const STAR: char = '*';
    pub const DASH: char = '-';
    pub const FWDSLASH: char = '/';
    pub const UNDERLINE: char = '_';
    pub const SINGLE_QUOTE: char = '\'';
    pub const DOT: char = '.';
    pub const HASH: char = '#';
    pub const COLON: char = ':';
    pub const PLUS: char = '+';

    /// Checks if `c` is a space according to VHDL-2008 LRM p225.
    /// Set: space, nbsp
    pub fn is_space(c: &char) -> bool {
        c == &'\u{0020}' || c == &'\u{00A0}'
    }

    /// Checks if `c` is a digit according to VHDL-2008 LRM p225.
    pub fn is_digit(c: &char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p230.
    /// - rule ::= upper_case_letter | digit | special_character | space_character 
    /// | lower_case_letter | other_special_character
    pub fn is_graphic(c: &char) -> bool {
        is_lower(&c) || is_upper(&c) || is_digit(&c) || 
        is_special(&c) || is_other_special(&c) || is_space(&c)
    }

    /// Checks if `c` is an upper-case letter according to VHDL-2019 LRM p257.
    /// Set: `ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ`
    pub fn is_upper(c: &char) -> bool {
        match c {
            '\u{00D7}' => false, // reject multiplication sign
            'A'..='Z' | 'À'..='Þ' => true,
            _ => false   
        }
    }

    /// Checks if `c` is a new-line character.
    pub fn is_newline(c: &char) -> bool {
        c == &'\n'
    }

    /// Checks if `c` is a special character according to VHDL-2008 LRM p225.
    /// Set: `"#&'()*+,-./:;<=>?@[]_`|`
    pub fn is_special(c: &char) -> bool {
        match c {
            '"' | '#' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | 
            ':' | ';' | '<'  | '=' | '>' | '?' | '@' | '[' | ']' | '_' | '`' | '|' => true,
            _ => false,
        }
    }

    /// Checks if `c` is an "other special character" according to VHDL-2008 LRM p225.
    /// Set: `!$%\^{} ~¡¢£¤¥¦§ ̈©a«¬® ̄°±23 ́μ¶· ̧1o»1⁄41⁄23⁄4¿×÷-`
    pub fn is_other_special(c: &char) -> bool {
        match c {
            '!' | '$' | '%' | '\\' | '^' | '{' | '}' | ' ' | '~' | '-' | 
            '\u{00A1}'..='\u{00BF}' | '\u{00D7}' | '\u{00F7}' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a lower-case letter according to VHDL-2019 LRM p257.
    /// Set: `abcdefghijklmnopqrstuvwxyzßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ`
    pub fn is_lower(c: &char) -> bool {
        match c {
            '\u{00F7}' => false, // reject division sign
            'a'..='z' | 'ß'..='ÿ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a letter according to VHDL-2019 LRM p257.
    pub fn is_letter(c: &char) -> bool {
        is_lower(&c) || is_upper(&c)
    }

    /// Checks if the character is a seperator according to VHDL-2019 LRM p259.
    pub fn is_separator(c: &char) -> bool {
        // whitespace: space, nbsp
        c == &'\u{0020}' || c == &'\u{00A0}' ||
        // format-effectors: ht (\t), vt, cr (\r), lf (\n)
        c == &'\u{0009}' || c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}'
    }
}

use std::str::FromStr;

/// Collects a basic identifer or a bit string literal with omitting integer.
/// - basic_identifier ::= letter { \[ underline ] letter_or_digit }
/// - bit_str_literal  ::= \[ integer ] base_specifier " \[ bit_value ] "
fn collect_identifier<T>(stream: &mut Peekable<T>, loc: &mut Position, c0: char) -> Result<VHDLToken, ()>
    where T: Iterator<Item=char> {

    let mut id = String::from(c0);
    let mut bit_lit: Option<BitStrLiteral> = None;
    let mut was_underline = false;

    while let Some(c) = stream.peek() {
        if (bit_lit.is_none() && (char_set::is_letter(&c) || c == &char_set::UNDERLINE || char_set::is_digit(&c))) ||
            (bit_lit.is_some() && c != &char_set::DOUBLE_QUOTE && (char_set::is_graphic(&c) || c == &char_set::UNDERLINE)) {
            // avoid double underline
            if c == &char_set::UNDERLINE && was_underline == true { panic!("cannot have double underline") }
            // remember if the current char was an underline for next state
            was_underline = c == &char_set::UNDERLINE;
            // consume character into literal/idenifier
            loc.next_col();
            id.push(stream.next().unwrap());
        // handle bit string literals 
        } else if c == &char_set::DOUBLE_QUOTE {
            if bit_lit.is_none() {
                let base = BaseSpec::from_str(&id)?;
                // clear id to begin reading string literal
                id.clear();
                // throw away initial " char
                loc.next_col();
                stream.next().unwrap(); 
                // enter creating a bit string literal
                // @TODO return Ok(collect_bit_str_literal(id, stream, loc))
                bit_lit = Some(BitStrLiteral::new(base));
            } else if bit_lit.is_some() {
                // verify the last character was not an underline
                if was_underline == true { panic!("last character cannot be underline") }
                // throw away closing " char
                loc.next_col();
                stream.next().unwrap(); 
                break; // exit loop
            }
        } else {
            if bit_lit.is_some() { panic!("missing closing quote") }
            break;
        }
    }
    match bit_lit {
        Some(b) => Ok(VHDLToken::BitStrLiteral(b.literal(id))),
        None => {
            // try to transform to key word
            Ok(match VHDLToken::match_keyword(&id) {
                Some(keyword) => keyword,
                None => VHDLToken::Identifier(Identifier::Basic(id))
            })
        }
    }
}

/// Collects a single-line comment (all characters after a `--` up until end-of-line).
fn collect_comment<T>(stream: &mut Peekable<T>, loc: &mut Position) -> VHDLToken
    where T: Iterator<Item=char> { 
    // skip over second '-'
    stream.next(); 
    loc.next_col();
    // consume characters to form the comment
    let mut note = String::new();
    while let Some(c) = stream.peek() {
        // cannot be vt, cr (\r), lf (\n)
        if c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}' {
            break
        } else {
            loc.next_col();
            note.push(stream.next().unwrap());
        }
    }
    VHDLToken::Comment(Comment::Single(note))
}

/// Captures the bit string literal.
/// 
/// At this point, the `value` will have (maybe) integer and a base_specifier.
/// - bit_string_literal ::=  \[ integer ] base_specifier " \[ bit_value ] "
fn collect_bit_str_literal<T>(value: String, stream: &mut Peekable<T>, loc: &mut Position) -> VHDLToken
where T: Iterator<Item=char> {

    todo!()
}

/// Collects a delimited comment (all characters after a `/*` up until `*/`).
fn collect_delim_comment<T>(stream: &mut Peekable<T>, loc: &mut Position) -> VHDLToken
    where T: Iterator<Item=char> { 
    // skip over opening '*'
    stream.next();
    loc.next_col();
    let mut note = String::new();
    while let Some(c) = stream.next() {
        loc.next_col();
        if char_set::is_newline(&c) == true {
            loc.next_line();
        }
        // check if we are breaking from the comment
        if c == char_set::STAR {
            if let Some(c_next) = stream.peek() {
                // break from the comment
                if c_next == &char_set::FWDSLASH {
                    loc.next_col();
                    stream.next();
                    break;
                }
            }
        }
        note.push(c);
    }
    VHDLToken::Comment(Comment::Delimited(note))
}

/// Captures an extended identifier token.
/// 
/// Errors if the identifier is empty.
fn collect_extended_identifier<T>(stream: &mut Peekable<T>, loc: &mut Position) -> Result<VHDLToken, ()>
where T: Iterator<Item=char> { 
    let id = enclose(&char_set::BACKSLASH, stream, loc);
    if id.is_empty() { panic!("extended identifier cannot be empty") }
    Ok(VHDLToken::Identifier(Identifier::Extended(id)))
}

/// Captures a character literal according to VHDL-2018 LRM p231.
fn collect_chr_lit<T>(stream: &mut Peekable<T>, loc: &mut Position) -> Result<VHDLToken, ()> 
where T: Iterator<Item=char> {
    let mut char_lit = String::with_capacity(1);
    if let Some(c) = stream.next() {
        // verify the character is a graphic character
        if char_set::is_graphic(&c) == false { panic!("invalid char {}", c) }
        loc.next_col();
        // add to the struct
        char_lit.push(c);
        // expect a closing single-quote 
        // @TODO handle errors
        if stream.next().expect("missing closing char") != char_set::SINGLE_QUOTE {
            panic!("expecting closing '\'' character")
        };
        loc.next_col();
    }
    Ok(VHDLToken::CharLiteral(Character(char_lit)))
}

/// Checks is a character `c` is within the given extended digit range set by `b`.
fn in_range(b: usize, c: &char) -> bool {
    let within_digit = (*c as usize) < char_set::ASCII_ZERO + b && (*c as usize) >= char_set::ASCII_ZERO;
    if b <= 10 {
        return within_digit
    } else {
        match b {
            11 => within_digit || match c { 'a'..='a' | 'A'..='A' => true, _ => false },
            12 => within_digit || match c { 'a'..='b' | 'A'..='B' => true, _ => false },
            13 => within_digit || match c { 'a'..='c' | 'A'..='C' => true, _ => false },
            14 => within_digit || match c { 'a'..='d' | 'A'..='D' => true, _ => false },
            15 => within_digit || match c { 'a'..='e' | 'A'..='E' => true, _ => false },
            16 => within_digit || match c { 'a'..='f' | 'A'..='F' => true, _ => false },
            _ => panic!("invalid base (only 2-16)")
        }
    }
}

/// Captures an abstract literal: either a decimal_literal or based_literal.
fn collect_abst_lit<T>(stream: &mut Peekable<T>, loc: &mut Position, c0: char) -> Result<VHDLToken, ()> 
where T: Iterator<Item=char> {
    // begin with first identified digit
    let mut lit = String::from(c0);
    // a base literal's base 
    let mut base: Option<usize> = None;
    // check if already used 'dot'
    let mut dotted = false; 
    // remember if last char was a digit 0..=9
    let mut was_digit = true; 
    // remember if the char is a ':' or '#' to start based literal
    let mut base_delim_char: Option<char> = None;
    // gather a base / number
    while let Some(c) = stream.peek() {
        // is a integer | underline | extended_digit
        if char_set::is_digit(&c) == true || c == &char_set::UNDERLINE || (base.is_some() && (c.is_ascii_alphabetic() || char_set::is_digit(&c))) {
            // verify character is within range for a based_literal
            if let Some(b) = base {
                if c != &char_set::UNDERLINE && in_range(b, &c) == false { panic!("invalid extended digit {} {}", b, c) }
            }
            if c == &char_set::UNDERLINE && was_digit == false { panic!("underline must come after a digit") }
            // remember if this char was a digit for next char logic
            was_digit = c != &char_set::UNDERLINE;
            loc.next_col();
            lit.push(stream.next().unwrap());
        // is a based_literal '#' char
        } else if c == &char_set::HASH || c == &char_set::COLON {
            // ensure we are using the right char
            if let Some(d) = base_delim_char {
                if c != &d { panic!("based literal must close with same character {}", d) }
            // remember the starting character
            } else {
                base_delim_char = Some(*c);
            }
            if was_digit == false { panic!("digit must come before hash") }
            // exit if it is the closing char '#'
            if base.is_some() {
                 // add char to lit
                loc.next_col();
                lit.push(stream.next().unwrap());
                break; // exit the loop
            }
            // convert lit to a base
            base = Some(lit.replace('_', "").parse::<usize>().unwrap());
            // verify the base is a good range
            if base < Some(2) || base > Some(16) { panic!("invalid base (2 <= x <= 16)") }
            // add char to lit
            loc.next_col();
            lit.push(stream.next().unwrap());
            was_digit = false;
        // is a dot '.' (decimal point)
        } else if c == &char_set::DOT {
            if dotted == true { panic!("cannot have multiple dots") };
            // verify the last char was a digit
            if was_digit == false { panic!("expected digit before dot") };
            // add dot to lit
            loc.next_col();
            lit.push(stream.next().unwrap());
            dotted = true;
            was_digit = false;
        } else {
            break;
        }
    }
    // check for exponent
    let has_exponent = if let Some(c) = stream.peek() {
        if c == &'e' || c == &'E' {
            loc.next_col();
            lit.push(stream.next().unwrap());
            true
        // pass to bit string literal
        } else if c.is_ascii_alphabetic() == true { 
            loc.next_col();
            let c = stream.next().unwrap();
            // @TODO somehow pass width found as `lit`?
            return collect_identifier(stream, loc, c)
        } else {
            false
        }
    } else { false };
    // capture exponent
    if has_exponent == true {
        // check for sign
        loc.next_col();
        let sign = stream.next().expect("missing exponent value");
        if sign != char_set::PLUS && sign != char_set::DASH && char_set::is_digit(&sign) == false {
            panic!("expecting +, -, or a digit")
        }
        was_digit = char_set::is_digit(&sign);
        lit.push(sign);
        while let Some(c) = stream.peek() {
            was_digit = if char_set::is_digit(&c) == true {
                loc.next_col();
                lit.push(stream.next().unwrap());
                true
            } else if c == &char_set::UNDERLINE {
                if was_digit == false { panic!("must have digit before underline")}
                loc.next_col();
                lit.push(stream.next().unwrap());
                false
            } else {
                if was_digit == false { panic!("must close with a digit") }
                break;
            }
        }
    }
    if base.is_some() {
        Ok(VHDLToken::AbstLiteral(AbstLiteral::Based(lit)))
    } else {
        Ok(VHDLToken::AbstLiteral(AbstLiteral::Decimal(lit)))
    }    
}

impl Tokenize for VHDLTokenizer {
    type TokenType = VHDLToken;

    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>> {
        let mut loc = Position::new();
        let mut chars = s.chars().peekable();
        // store results here as we consume the characters
        let mut tokens = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = chars.next() {
            loc.next_col();

            let tk_loc = Position(loc.0, loc.1);
            if char_set::is_letter(&c) {
                // collect general identifier (or bit string literal) 
                let tk = collect_identifier(&mut chars, &mut loc, c).expect("failed to read identifier");
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::BACKSLASH {
                // collect extended identifier
                let tk = collect_extended_identifier(&mut chars, &mut loc).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::DOUBLE_QUOTE {
                // collect string literal
                let tk = VHDLToken::StrLiteral(enclose(&c, &mut chars, &mut loc));
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::SINGLE_QUOTE {
                // collect character literal
                let tk = collect_chr_lit(&mut chars, &mut loc).expect("invalid char literal");
                tokens.push(Token::new(tk, tk_loc));

            } else if char_set::is_digit(&c) {
                // collect decimal literal (or bit string literal or based literal)
                let tk = collect_abst_lit(&mut chars, &mut loc, c).expect("invalid abst literal");
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::DASH && chars.peek().is_some() && chars.peek().unwrap() == &char_set::DASH {    
                // collect a single-line comment           
                let tk = collect_comment(&mut chars, &mut loc);
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::FWDSLASH && chars.peek().is_some() && chars.peek().unwrap() == &char_set::STAR {
                // collect delimited (multi-line) comment
                let tk = collect_delim_comment(&mut chars, &mut loc);
                tokens.push(Token::new(tk, tk_loc));

            } else {
                // collect delimiter
                if let Some(tk) = collect_delimiter(&mut chars, &mut loc, Some(c)) {
                    tokens.push(Token::new(tk, tk_loc));
                }
            }
            // o.w. collect whitespace
            if char_set::is_newline(&c) == true {
                loc.next_line();
            }
        }
        // push final EOF token
        loc.next_col();
        tokens.push(Token::new(VHDLToken::EOF, loc));
        tokens
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ignore_case_cmp() {
        let s0 = "ABC";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), true);

        // negative case: different lengths
        let s0 = "ABCD";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // negative case: different letter order
        let s0 = "cba";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // VHDL-2008 LRM p226
        let s0 = "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ";
        let s1 = "abcdefghijklmnopqrstuvwxyzàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþ";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // these 2 letters do not have upper-case equivalents
        let s0 = "ß";
        let s1 = "ÿ";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);
    }

    mod vhdl {
        use super::*;

        #[test]
        fn read_deci_literal() {
            let mut loc = Position(1, 1);
            let contents = "234";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Decimal("1234".to_owned())));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 4));
        }
    
        #[test]
        fn read_deci_literal_2() {
            let mut loc = Position(1, 1);
            let contents = "23_4.5;";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Decimal("123_4.5".to_owned())));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 7));
        }

        #[test]
        #[ignore]
        fn read_full_bit_str_literal() {
            let mut loc = Position(1, 1);
            let contents = "0b\"10_1001_1111\";";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::BitStrLiteral(vhdl::BitStrLiteral::new(BaseSpec::B).literal("10_1001_1111".to_owned()).width(10)));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 17));

            let mut loc = Position(1, 1);
            let contents = "2SX\"F-\";";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::BitStrLiteral(vhdl::BitStrLiteral::new(BaseSpec::SX).literal("F-".to_owned()).width(12)));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 8));
        }

        #[test]
        fn read_deci_literal_exp() {
            let mut loc = Position(1, 1);
            let contents = ".023E+24";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '6').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Decimal("6.023E+24".to_owned())));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 9));

            let mut loc = Position(1, 1);
            let contents = "E6";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Decimal("1E6".to_owned())));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 3));

            let mut loc = Position(1, 1);
            let contents = ".34e-12;";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Decimal("1.34e-12".to_owned())));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 8));
        }

        #[test]
        fn read_based_literal() {
            let mut loc = Position(1, 1);
            let contents = "#1001_1010#;";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '2').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Based("2#1001_1010#".to_owned())));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 12));

            let mut loc = Position(1, 1);
            let contents = "6#abcd_FFFF#;";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Based("16#abcd_FFFF#".to_owned())));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 13));

            // colon ':' can be replacement if used as open and closing VHDL-2019 LRM p180
            let mut loc = Position(1, 1);
            let contents = "6:abcd_FFFF:;";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_abst_lit(&mut stream, &mut loc, '1').unwrap(), VHDLToken::AbstLiteral(vhdl::AbstLiteral::Based("16:abcd_FFFF:".to_owned())));
            assert_eq!(stream.collect::<String>(), ";");
            assert_eq!(loc, Position(1, 13));
        }

        #[test]
        fn easy_tokens() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.take() })
                .collect();
            assert_eq!(tokens, vec![
                Entity,
                Identifier(vhdl::Identifier::Basic("fa".to_owned())),
                Is,
                End,
                Entity,
                Terminator,
                EOF,
            ]);
        }

        #[test]
        fn comment_token() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
-- here is a vhdl single-line comment!";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Single(" here is a vhdl single-line comment!".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(1, 39)),
            ]);
        }

        #[test]
        fn comment_token_delim() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
/* here is a vhdl 
    delimited-line comment. Look at all the space! */";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Delimited(" here is a vhdl 
    delimited-line comment. Look at all the space! ".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(2, 54)),
            ]);
        }

        #[test]
        fn char_literal() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
signal magic_num : std_logic := '1';";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Signal, Position(1, 1)),
                Token::new(Identifier(vhdl::Identifier::Basic("magic_num".to_owned())), Position(1, 8)),
                Token::new(Colon, Position(1, 18)),
                Token::new(Identifier(vhdl::Identifier::Basic("std_logic".to_owned())), Position(1, 20)),
                Token::new(VarAssign, Position(1, 30)),
                Token::new(CharLiteral(vhdl::Character("1".to_owned())), Position(1, 33)),
                Token::new(Terminator, Position(1, 36)),
                Token::new(EOF, Position(1, 37)),
            ]);
        }

        #[test]
        fn easy_locations() {
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<Position> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.locate().clone() })
                .collect();
            assert_eq!(tokens, vec![
                Position(1, 1),  // 1:1 keyword: entity
                Position(1, 8),  // 1:8 basic identifier: fa
                Position(1, 11), // 1:11 keyword: is
                Position(1, 14), // 1:14 keyword: end
                Position(1, 18), // 1:18 keyword: entity
                Position(1, 24), // 1:24 delimiter: ;
                Position(1, 25), // 1:25 eof
            ]);  
        }

        #[test]
        fn read_delimiter_single() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "&";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Ampersand));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 1));

            let mut loc = Position::new();
            let contents = "?";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Question));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 1));

            let mut loc = Position::new();
            let contents = "< MAX_COUNT";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Lt));
            assert_eq!(stream.collect::<String>(), " MAX_COUNT");
            assert_eq!(loc, Position(1, 1));
        }

        #[test]
        fn read_delimiter_none() {
            let mut loc = Position::new();
            let contents = "fa";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), None);
            assert_eq!(stream.collect::<String>(), "fa");
            assert_eq!(loc, Position(1, 0));
        }

        #[test]
        fn read_delimiter_double() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "<=";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(SigAssign));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 2));

            let mut loc = Position::new();
            let contents = "**WIDTH";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(DoubleStar));
            assert_eq!(stream.collect::<String>(), "WIDTH");
            assert_eq!(loc, Position(1, 2));
        }

        #[test]
        fn read_delimiter_triple() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "<=>";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(SigAssoc));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 3));

            let mut loc = Position::new();
            let contents = "?/= MAGIC_NUM";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(MatchNE));
            assert_eq!(stream.collect::<String>(), " MAGIC_NUM");
            assert_eq!(loc, Position(1, 3));
        }

        #[test]
        fn match_delimiter() {
            use super::VHDLToken::*;

            let word = "<=";
            assert_eq!(VHDLToken::match_delimiter(word), Some(SigAssign));

            let word = "-";
            assert_eq!(VHDLToken::match_delimiter(word), Some(Dash));

            let word = "<=>";
            assert_eq!(VHDLToken::match_delimiter(word), Some(SigAssoc));

            let word = "^";
            assert_eq!(VHDLToken::match_delimiter(word), None);

            let word = "entity";
            assert_eq!(VHDLToken::match_delimiter(word), None);
        }

        #[test]
        fn match_reserved_idenifier() {
            use super::VHDLToken::*;

            let word = "END";
            assert_eq!(VHDLToken::match_keyword(word), Some(End));

            let word = "EnTITY";
            assert_eq!(VHDLToken::match_keyword(word), Some(Entity));

            let word = "entitys";
            assert_eq!(VHDLToken::match_keyword(word), None);

            let word = "<=";
            assert_eq!(VHDLToken::match_keyword(word), None);
        }

        #[test]
        fn is_sep() {
            let c = ' '; // space
            assert_eq!(char_set::is_separator(&c), true);

            let c = '\u{00A0}'; // nbsp
            assert_eq!(char_set::is_separator(&c), true);

            let c = '\t'; // horizontal tab
            assert_eq!(char_set::is_separator(&c), true);

            let c = '\n'; // new-line
            assert_eq!(char_set::is_separator(&c), true);

            let c = 'c';  // negative case: ascii char
            assert_eq!(char_set::is_separator(&c), false);
        }

        #[test]
        fn read_identifier() {
            let mut loc = Position(1, 1);
            let words = "ntity is";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'e').unwrap(), VHDLToken::Entity);
            assert_eq!(stream.collect::<String>(), " is");
            assert_eq!(loc, Position(1, 6));

            let mut loc = Position(1, 1);
            let words = "td_logic_1164.all;";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 's').unwrap(), VHDLToken::Identifier(vhdl::Identifier::Basic("std_logic_1164".to_owned())));
            assert_eq!(stream.collect::<String>(), ".all;");
            assert_eq!(loc, Position(1, 14));

            let mut loc = Position(1, 1);
            let words = "eady_OUT<=";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'r').unwrap(), VHDLToken::Identifier(vhdl::Identifier::Basic("ready_OUT".to_owned())));
            assert_eq!(stream.collect::<String>(), "<=");
            assert_eq!(loc, Position(1, 9));
        }

        #[test]
        fn read_bit_str_literal() {
            let mut loc = Position(1, 1);
            let words = "\"1010\"more text";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'b'), Ok(VHDLToken::BitStrLiteral(vhdl::BitStrLiteral { width: None, base: BaseSpec::B, literal: "1010".to_owned() })));
            assert_eq!(stream.collect::<String>(), "more text");
            assert_eq!(loc, Position(1, 7));
        }
        
        #[test]
        fn eq_identifiers() {
            let id0 = Identifier::Basic("fa".to_owned());
            let id1 = Identifier::Basic("Fa".to_owned());
            assert_eq!(id0, id1);

            let id0 = Identifier::Basic("fa".to_owned());
            let id1 = Identifier::Basic("Full_adder".to_owned());
            assert_ne!(id0, id1);

            let id0 = Identifier::Basic("VHDL".to_owned());    // written as: VHDL
            let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
            assert_ne!(id0, id1);

            let id0 = Identifier::Extended("vhdl".to_owned()); // written as: \vhdl\
            let id1 = Identifier::Extended("VHDL".to_owned()); // written as: \VHDL\
            assert_ne!(id0, id1);
        }

        #[test]
        fn wrap_enclose() {
            let mut loc = Position(1, 1);
            let contents = "\"Setup time is too short\"more text";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "Setup time is too short");
            assert_eq!(stream.collect::<String>(), "more text");
            assert_eq!(loc, Position(1, 25));

            let mut loc = Position(1, 1);
            let contents = "\"\"\"\"\"\"";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "\"\"");
            assert_eq!(loc, Position(1, 6));

            let mut loc = Position::new();
            let contents = "\" go \"\"gators\"\" from UF! \"";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), " go \"gators\" from UF! ");
            assert_eq!(loc, Position(1, 25));

            let mut loc = Position::new();
            let contents = "\\VHDL\\";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "VHDL");

            let mut loc = Position::new();
            let contents = "\\a\\\\b\\more text afterward";
            let mut stream = contents.chars().peekable();
            let br = stream.next().unwrap();
            assert_eq!(enclose(&br, &mut stream, &mut loc), "a\\b");
            // verify the stream is left in the correct state
            assert_eq!(stream.collect::<String>(), "more text afterward");
        }

        #[test]
        #[ignore]
        fn nor_gate_design_code() {
            let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity nor_gate is 
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        \\In\\ : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
end entity nor_gate;

architecture rtl of nor_gate is
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 8x\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl;";
            let vhdl = VHDLTokenizer::tokenize(&s);
            let vhdl = VHDLTokenizer { inner: vhdl };
            println!("{:?}", vhdl);
            todo!()
        }
    }

    mod position {
        use super::*;

        #[test]
        fn moving_position() {
            let mut pos = Position::new();
            assert_eq!(pos, Position(1, 0));
            pos.next_col();
            assert_eq!(pos, Position(1, 1));
            pos.next_col();
            assert_eq!(pos, Position(1, 2));
            pos.next_line();
            assert_eq!(pos, Position(2, 0));
            pos.next_line();
            assert_eq!(pos, Position(3, 0));
        }
    }
}