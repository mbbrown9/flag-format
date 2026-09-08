use crate::model::FlagFile;

pub fn pretty_print(file: &FlagFile) -> String {
    let mut flags = file.flags.clone();
    flags.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = String::new();
    for (i, flag) in flags.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str("flag ");
        out.push_str(&flag.name);
        out.push_str(" {\n");
        out.push_str(&format!("    enabled = {}\n", flag.enabled));
        if let Some(desc) = &flag.description {
            out.push_str(&format!("    description = {}\n", quote(desc)));
        }
        if let Some(rollout) = flag.rollout {
            out.push_str(&format!("    rollout = {}%\n", rollout));
        }
        if !flag.tags.is_empty() {
            let mut tags = flag.tags.clone();
            tags.sort();
            out.push_str(&format!("    tags = [{}]\n", tags.join(", ")));
        }
        out.push_str("}\n");
    }
    out
}

pub fn to_json(file: &FlagFile) -> String {
    let mut flags = file.flags.clone();
    flags.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = String::new();
    out.push_str("{\n  \"flags\": [\n");
    for (i, flag) in flags.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_string(&flag.name)));
        out.push_str(&format!("      \"enabled\": {},\n", flag.enabled));
        out.push_str(&format!(
            "      \"description\": {},\n",
            match &flag.description {
                Some(d) => json_string(d),
                None => "null".to_string(),
            }
        ));
        out.push_str(&format!(
            "      \"rollout\": {},\n",
            match flag.rollout {
                Some(r) => r.to_string(),
                None => "null".to_string(),
            }
        ));
        let mut tags = flag.tags.clone();
        tags.sort();
        let tags_json: Vec<String> = tags.iter().map(|t| json_string(t)).collect();
        out.push_str(&format!("      \"tags\": [{}]\n", tags_json.join(", ")));
        out.push_str("    }");
        if i + 1 < flags.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
