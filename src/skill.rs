use std::{
    fs::File,
    io::{self, Read},
};

use crate::yaml::{self, Scalar};

fn find_three_dashes(buf: &[u8]) -> usize {
    for i in 0..buf.len() - 2 {
        if buf[i] != b'-' {
            continue;
        }

        if buf[i + 1] == b'-' && buf[i + 2] == b'-' {
            return i;
        }
    }

    unreachable!()
}

pub(crate) struct Skill {
    pub(crate) name: String,
    pub(crate) description: String,
    body: String,
}

impl Skill {
    pub(crate) fn new_from_filepath(filepath: &str) -> Self {
        let mut file = File::open(filepath).unwrap();
        let mut buffer: Vec<u8> = vec![];
        file.read_to_end(&mut buffer).unwrap();

        Self::new_from_bytes(&buffer)
    }

    pub(crate) fn new_from_bytes(buffer: &[u8]) -> Self {
        let first_dashes_end = find_three_dashes(&buffer[..]) + 4;
        let second_dashes_dist = find_three_dashes(&buffer[first_dashes_end..]) + first_dashes_end;

        let map = yaml::Parser::parse(&buffer[first_dashes_end..second_dashes_dist])
            .unwrap()
            .into_map();

        Self {
            name: map.get("name").unwrap().into_string(),
            description: map.get("description").unwrap().into_string(),
            body: String::new(),
        }
    }
}

pub(crate) fn load_all_skill_files() -> Result<Vec<Skill>, io::Error> {
    Ok(std::fs::read_dir("./.claude/skills/")?
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                return None;
            }

            Some(Skill::new_from_filepath(
                entry.path().join("SKILL.md").to_str().unwrap(),
            ))
        })
        .collect())
}

pub(crate) fn compile_all_skills_message_content() -> Option<String> {
    match load_all_skill_files() {
        Err(_) => None,
        Ok(skills) => {
            if skills.is_empty() {
                return None;
            }

            let mut out = String::from("You have access to the following skills:\n\n");

            skills.iter().for_each(|skill| {
                out.push_str(&format!("- {}: {}\n", skill.name, skill.description))
            });

            Some(out)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::skill::Skill;

    #[test]
    fn test_basic() {
        let result = Skill::new_from_bytes(
            b"  \n---\nname: The name\ndescription: The description\n---  \n  ",
        );
    }
}
