use std::{collections::HashMap, fs::File, io::Read};

use crate::yaml::{self};

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
    pub(crate) body: String,
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
        let second_dashes_start = find_three_dashes(&buffer[first_dashes_end..]) + first_dashes_end;

        let map = yaml::Parser::parse(&buffer[first_dashes_end..second_dashes_start])
            .unwrap()
            .into_map();

        let body = String::from_utf8(buffer[second_dashes_start + 5..].to_vec()).unwrap();

        Self {
            name: map.get("name").unwrap().into_string(),
            description: map.get("description").unwrap().into_string(),
            body,
        }
    }

    pub(crate) fn body_with_args_embed(&self, args: &[&str]) -> String {
        let mut out = self.body.clone();

        for (i, arg) in args.iter().enumerate() {
            let pattern = format!("$ARGUMENTS[{}]", i);
            let shorthand_pattern = format!("${}", i);

            out = out.replace(&pattern, arg);
            out = out.replace(&shorthand_pattern, arg);
        }

        let full_args = args.join(" ");
        out = out.replace("$ARGUMENTS", &full_args);

        return out;
    }
}

pub(crate) fn load_all_skill_files() -> HashMap<String, Skill> {
    let mut map = HashMap::new();

    match std::fs::read_dir("./.claude/skills/") {
        Ok(dirs) => {
            dirs.for_each(|entry| {
                let entry = entry.unwrap();
                if !entry.file_type().unwrap().is_dir() {
                    return;
                }

                let skill =
                    Skill::new_from_filepath(entry.path().join("SKILL.md").to_str().unwrap());

                map.insert(skill.name.clone(), skill);
            });
        }
        _ => {}
    }

    map
}

pub(crate) fn compile_all_skills_message_content(skills: &HashMap<String, Skill>) -> String {
    let mut out = String::from("You have access to the following skills:\n\n");

    skills
        .iter()
        .for_each(|(_, skill)| out.push_str(&format!("- {}: {}\n", skill.name, skill.description)));

    out.push_str("\nIf a skill matches the user's request, call the Skill tool with its name and follow the instructions it returns.\n");

    out
}

#[cfg(test)]
mod test {
    use crate::skill::Skill;

    #[test]
    fn test_basic() {
        let _result = Skill::new_from_bytes(
            b"  \n---\nname: The name\ndescription: The description\n---  \n  ",
        );
    }
}
