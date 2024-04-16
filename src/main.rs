use std::collections::{HashMap, HashSet};

use regex::Regex;

const LEFT_QUOTE : char = '`';
const RIGHT_QUOTE : char = '\'';

#[derive(PartialEq)]
enum TokenType {
    NAME,
    STRING,
    LITERAL,
    END
}

#[derive(PartialEq)]
enum TokenReadType {
    TOKEN,
    NAME,
    STRING
}

struct Token {
    type_: TokenType,
    value: String
}

#[derive(Clone)]
struct Macro {
    name: String,
    value: String,
    args: Vec<String>,
    parens: i32
}

impl Macro {
    fn new(name: &String, value: &String) -> Self {
        let args = vec![name.clone()];
        Self {
            name: name.clone(),
            value: value.clone(),
            args: args,
            parens: 0
        }
    }
}

fn expand(val: &String, args: &Vec<String>) -> String{
    let re = Regex::new(r"\$[0-9]+").unwrap();
    let matches: HashSet<_> = re.find_iter(val)
                                .map(|m| m.as_str())
                                .collect();
    
    let mut result = val.clone();
    for &idx in matches.iter() {
        if let Some(idx_int) = idx[1..].parse::<usize>().ok() {
            result = result.replace(idx, &args[idx_int]);
        }
    }
    result
}

struct Tokenizer{
    source: String,
    source_idx: usize,
    buffer: String,
    quotes: usize,
    end: bool,
    read_type: TokenReadType
}

fn is_literal_char(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_')
}

impl Tokenizer {
    fn new() -> Self{
        Self { 
            source: "".to_owned(), 
            source_idx: 0, 
            buffer: "".to_owned(), 
            quotes: 0, 
            end: false,
            read_type: TokenReadType::TOKEN
        }
    }

    fn flush(&mut self){
        self.source = self.source[self.source_idx..].to_owned();
        self.source_idx = 0
    }

    fn push(&mut self, st: &str){
        self.flush();
        self.source.push_str(st);
    }

    fn unshift(&mut self, st: &str) {
        self.flush();
        let mut st = st.to_owned();
        st.push_str(&self.source);
        self.source = st;
    }

    fn peek_char(&self) -> Option<char> {
        if self.source.len() == 0 {
            return None
        }
        self.source.chars().nth(self.source_idx)
    }

    fn read(&mut self) -> Option<Token> {
        while self.source_idx < self.source.len() {
            let Some(c) = self.source.chars().nth(self.source_idx) else {
                self.source_idx += 1;
                continue;
            };

            if let Some(token) = self.read_next(&Some(c)) {
                return Some(token);
            }
            self.source_idx += 1;
        }

        if !self.end {
            return None;
        }

        return self.read_next(&None);
    }

    fn read_next(&mut self, c: &Option<char>) -> Option<Token> {
        match self.read_type {
            TokenReadType::TOKEN => self.read_token(c),
            TokenReadType::NAME => self.read_name(c),
            TokenReadType::STRING => self.read_string(c),
        }
    }

    fn get_token(&mut self, type_: TokenType) -> Token {
        let buffer = self.buffer.clone();
        self.read_type = TokenReadType::TOKEN;
        self.buffer = "".to_owned();
        Token{
            type_: type_,
            value: buffer
        }
    }

    fn read_token(&mut self, c: &Option<char>) -> Option<Token> {
        let Some(c) = c else {
            self.end = false;
            return Some(self.get_token(TokenType::END));
        };
        let c = c.clone();
        self.buffer.push(c);

        if is_literal_char(c) {
            self.read_type = TokenReadType::NAME;
            return None
        }

        if c == LEFT_QUOTE {
            self.read_type = TokenReadType::STRING;
            self.quotes += 1;
            self.buffer = "".to_owned();
            return None;
        }
        
        self.source_idx += 1;
        Some(self.get_token(TokenType::LITERAL))
        
    }

    fn read_name(&mut self, c: &Option<char>) -> Option<Token> {
        let Some(c) = c else {
            return Some(self.get_token(TokenType::NAME));
        };

        let c = c.clone();
        if !is_literal_char(c) {
            return Some(self.get_token(TokenType::NAME));
        }

        self.buffer.push(c);

        None
    }

    fn read_string(&mut self, c: &Option<char>) -> Option<Token> {
        let Some(c) = c else {
            return None
        };
        let c = c.clone();
        self.buffer.push(c);
        if c == LEFT_QUOTE {
            self.quotes += 1;
        }
        else if c == RIGHT_QUOTE {
            self.quotes -= 1;
            if self.quotes == 0 {
                self.source_idx += 1;
                self.buffer = self.buffer[..self.buffer.len()-1].to_owned();
                return Some(self.get_token(TokenType::STRING));
            }
        }
        None
    }
}

struct M4 {
    pending: Option<Macro>,
    macro_stack: Vec<Macro>,
    skip_whitespace: bool,
    output: String,
    tokenizer: Tokenizer,
    macros: HashMap<String, String>
}

impl M4 {
    fn new() -> Self {
        Self {
            pending: None,
            macro_stack: Vec::new(),
            skip_whitespace: false,
            output: "".to_owned(),
            tokenizer: Tokenizer::new(),
            macros: HashMap::new()
        }
    }

    fn define(&mut self, name: &String, value: &String) {
        self.macros.insert(name.clone(), value.clone());
    }

    fn call_macro(&mut self, m: &Macro) -> String {
        if m.name == "define" {
            self.define(&m.args[1], &m.args[2]);
            return "".to_owned();
        }
        expand(&m.value, &m.args)
    }

    fn process_pending_macro(&mut self) {
        if self.pending.is_none() {
            return;
        }

        let Some(ch) = self.tokenizer.peek_char() else {
            return;
        };

        if ch == '('{
            return self.start_macro_args();
        }
        
        if let Some(m) = self.pending.as_ref() {
            let result = self.call_macro(&m.clone());
            self.tokenizer.unshift(&result);
            self.pending = None;
        }
    }

    fn start_macro_args(&mut self) {
        self.tokenizer.read();
        if self.pending.is_none() {
            return;
        }

        if let Some(m) = self.pending.as_mut() {
            m.args.push("".to_owned());
            self.macro_stack.push(m.clone());
            self.pending = None;
            self.skip_whitespace = true;
        }
    }

    fn process_token(&mut self, token:&Token) {
        if self.skip_whitespace && token.type_ == TokenType::LITERAL 
            && token.value.trim().is_empty() 
        {
            return;
        }

        self.skip_whitespace = false;

        if token.type_ == TokenType::NAME && (
            self.macros.contains_key(&token.value) || token.value == "define"
        ) {
            self.pending = Some(
                Macro::new(
                    &token.value, 
                    self.macros.get(&token.value)
                        .unwrap_or(&"".to_owned())
                        
                )
            );
            return;
        }

        if self.macro_stack.len() == 0 {
            self.push_output(&token.value);
            return;
        }


        if token.type_ == TokenType::LITERAL {
            if self.process_literal_in_macro(&token) {
                return;
            }
        }
        
        if let Some(m) = self.macro_stack.last_mut() {
            if let Some(arg) = m.args.last_mut() {
                arg.push_str(&token.value);
            }
        }
    }

    fn push_output(&mut self, output: &String) {
        self.output.push_str(output)
    }

    fn process_literal_in_macro(&mut self, token: &Token) -> bool {
        let Some(m) = self.macro_stack.last_mut() else{
            return false;
        };
        let m = m.clone();

        if token.value == ")" {
            if m.parens == 0 {
                self.macro_stack.pop();
                let result = self.call_macro(&m);
                self.tokenizer.unshift(&result);
                return true;
            }
            if let Some(m) = self.macro_stack.last_mut() {
                m.parens -= 1;
            }
        }else if token.value == "(" {
            if let Some(m) = self.macro_stack.last_mut(){
                m.parens += 1;
            }
        }else if token.value == "," && m.parens == 0 {
            if let Some(m) = self.macro_stack.last_mut(){
                m.args.push("".to_owned());
                self.skip_whitespace = true;
                return true;
            }
        }
        false
    }

    fn write(&mut self, chunk: &str) {
        self.tokenizer.push(&chunk);
        self.process_pending_macro();
        let mut token = self.tokenizer.read();
        while token.is_some() {
            let Some(token_ref) = token.as_ref() else {
                continue;
            };
            self.process_token(token_ref);
            self.process_pending_macro();
            token = self.tokenizer.read();
        }

    }
}

static INPUTS: [&str;2] =
[
"
define(`foo', `Hello world.')\n\
foo\n\
",
"
define(`exch', `$2, $1')\n\
exch(arg1, arg2)\n\
"
];

fn main() {
    for i in 0..2 {
        println!("===============================");
        let mut m4 = M4::new();
        println!("Input macro:");
        println!("{}", INPUTS[i]);
        m4.write(INPUTS[i]);
        print!("Output result:");
        println!("{}", m4.output);
    }
}
