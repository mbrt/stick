use std::ascii::AsciiExt;
use regex::Regex;

pub struct TitleParser {
    re: Regex,
}

#[derive(Debug,Eq,PartialEq)]
pub struct Title {
    pub descr: String,
    pub id: Option<String>,
}


impl TitleParser {
    pub fn new() -> Self {
        TitleParser { re: Regex::new(r"^([\dA-Za-z-]*)\s*[-:_]?\s*(.*)").unwrap() }
    }

    pub fn parse(&self, fname: &str, line: &str) -> Title {
        match self.re.captures(line) {
            None => {
                Title {
                    descr: line.to_owned(),
                    id: None,
                }
            }
            Some(captures) => {
                let id = captures.at(1).unwrap();
                let title = captures.at(2).unwrap();
                if id.to_ascii_lowercase() == fname.to_ascii_lowercase() {
                    Title {
                        descr: title.to_owned(),
                        id: Some(id.to_owned()),
                    }
                } else {
                    Title {
                        descr: line.to_owned(),
                        id: None,
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ok() {
        let parser = TicketParser::new();
        assert_eq!(parser.parse("id-151", "ID-151 My title"),
                   Title {
                       descr: "My title".to_owned(),
                       id: Some("ID-151".to_owned()),
                   });
        assert_eq!(parser.parse("id-151", "id-151 - My title"),
                   Title {
                       descr: "My title".to_owned(),
                       id: Some("id-151".to_owned()),
                   });
    }

    #[test]
    fn parse_fail() {
        let parser = TicketParser::new();
        assert_eq!(parser.parse("id-151", "ID-140 My title"),
                   Title {
                       descr: "ID-140 My title".to_owned(),
                       id: None,
                   });
    }
}
