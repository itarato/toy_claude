use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Scalar {
    String(String),
    Map(HashMap<String, Scalar>),
}

impl Scalar {
    pub(crate) fn into_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            _ => panic!("Expected scalar string"),
        }
    }

    pub(crate) fn into_map(&self) -> HashMap<String, Scalar> {
        match self {
            Self::Map(m) => m.clone(),
            _ => panic!("Expected scalar map"),
        }
    }
}

pub(crate) struct Parser {}

impl Parser {
    pub(crate) fn parse(source: &[u8]) -> Result<Scalar, String> {
        assert!(!source.is_empty());
        assert!(source[0].is_ascii_alphabetic());

        match Self::parse_dict(source) {
            Ok((_new_source, map)) => return Ok(map),
            Err(err) => {
                panic!(
                    "Failed YAML parsing. Error = {:?}\n\nYAML:\n\n{:?}\n\n",
                    err,
                    String::from_utf8(source.to_vec()).unwrap()
                )
            }
        };
    }

    fn parse_dict(mut source: &[u8]) -> Result<(&[u8], Scalar), String> {
        let mut map = HashMap::new();

        loop {
            if source.is_empty() {
                break;
            }

            let (new_source, key) = Self::parse_word(source)?;
            source = new_source;

            match source[0] {
                b':' => {
                    source = &source[2..];
                    let (new_source, value) = Self::parse_till_eol(source)?;
                    source = &new_source[1..];

                    map.insert(key, Scalar::String(value));
                }
                other => unimplemented!("Dict parser found unexpected char: {}", other),
            }
        }

        Ok((source, Scalar::Map(map)))
    }

    fn parse_word(source: &[u8]) -> Result<(&[u8], String), String> {
        let res: Vec<u8> = source
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric())
            .cloned()
            .collect::<Vec<_>>();
        let out_source = &source[res.len()..];
        let out = String::from_utf8(res).unwrap();

        Ok((out_source, out))
    }

    fn parse_till_eol(source: &[u8]) -> Result<(&[u8], String), String> {
        let res: Vec<u8> = source
            .iter()
            .take_while(|c| **c != b'\n')
            .cloned()
            .collect::<Vec<_>>();
        let out_source = &source[res.len()..];
        let out = String::from_utf8(res).unwrap();

        Ok((out_source, out))
    }
}

#[cfg(test)]
mod test {
    use crate::yaml::{Parser, Scalar};

    #[test]
    fn test_basic() {
        let result = Parser::parse(b"name: The name\ndescription: The description\n").unwrap();

        match result {
            Scalar::Map(map) => {
                assert_eq!(
                    map.get("name").unwrap(),
                    &Scalar::String(String::from("The name"))
                );
                assert_eq!(
                    map.get("description").unwrap(),
                    &Scalar::String(String::from("The description"))
                );
            }
            _ => assert!(false),
        }
    }
}
