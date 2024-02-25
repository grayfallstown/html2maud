use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;
use tl::*;

fn format_empty_blocks(s: &str) -> String {
    let re = Regex::new(r"(?m)\{\s*\}").unwrap();
    re.replace_all(s, "{}").into_owned()
}

fn convert_hash_id_to_id_attribute(input: &str) -> String {
    let re = Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap();
    let mut id_counts = HashMap::new();

    // Count occurrences of each #id
    for caps in re.captures_iter(input) {
        let id = caps.get(1).unwrap().as_str();
        *id_counts.entry(id.to_string()).or_insert(0) += 1;
    }

    // Collect the keys into a vector to avoid borrowing issues
    let ids: Vec<String> = id_counts.keys().cloned().collect();

    // Additionally, count occurrences of id in other contexts
    for id in ids {
        let re_id = Regex::new(&format!(r#"[^\#]{}[^\-]"#, id)).unwrap();
        for _ in re_id.find_iter(input) {
            *id_counts.get_mut(&id).unwrap() += 1;
        }
    }

    // Replace #id with id="id" only if id occurs more than once
    let result = re.replace_all(input, |caps: &regex::Captures| {
        let id = caps.get(1).unwrap().as_str();
        if id_counts.get(id).unwrap_or(&0) > &1 {
            format!(r#"id="{}""#, id)
        } else {
            format!("#{}", id)
        }
    });

    result.into_owned()
}

fn remove_empty_lines(s: &str) -> String {
    let lines = s.lines();
    let non_empty_lines: Vec<&str> = lines.filter(|line| line.trim().len() > 0).collect();
    convert_hash_id_to_id_attribute(&format_empty_blocks(&non_empty_lines.join("\n")))
}

pub fn html2maud(html: &str) -> String {
    let mut maud_template = String::new();

    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    fn spaces(count: usize) -> String {
        return "    ".repeat(count).as_str().to_owned();
    }

    fn handle_tag(tag: &HTMLTag, parser: &Parser, maud_template: &mut String, indent: usize) {
        let tag_name = tag.name().as_utf8_str();

        let use_semicolon = match tag_name.as_ref().to_string().as_str() {
            "meta" | "link" | "br" | "img" | "input" | "hr" | "col" | "area" | "base" | "wbr"
            | "track" | "param" => true,
            _ => false,
        };

        write!(maud_template, "{}{}", spaces(indent), &tag_name).unwrap();

        match tag.attributes().class_iter() {
            None => {}
            Some(classes) => {
                write!(maud_template, ".\"").unwrap();
                let mut class_strings = Vec::new();
                for class in classes {
                    let escaped_class = if class.contains("-") {
                        format!("{}", &class)
                    } else {
                        class.to_owned()
                    };
                    class_strings.push(escaped_class);
                }
                let classes_str = class_strings.join(" ");
                write!(maud_template, "{}\"", classes_str).unwrap();
            }
        }

        let id = tag.attributes().id().map(|x| x.as_utf8_str());
        match &id {
            Option::Some(x) => {
                let escaped_id = if x.contains("-") {
                    /* format!("\"{}\"", &x) */
                    format!("{}", &x)
                } else {
                    x.to_string()
                };
                //write!(maud_template, " id=\"{}\"", &escaped_id).unwrap();
                write!(maud_template, " #{}", &escaped_id).unwrap();
            }
            Option::None => {}
        }

        for (key, value_opt) in tag.attributes().iter() {
            if !(key.eq("id") || key.eq("class")) {
                write!(maud_template, " {}", key).unwrap();
                match value_opt {
                    None => {}
                    Some(value) => write!(maud_template, "=\"{}\"", value).unwrap(),
                }
            }
        }

        if !use_semicolon {
            write!(maud_template, " {{\n").unwrap();
        } else {
            write!(maud_template, ";\n").unwrap();
        }

        let children = tag.children();
        let nodes = children.top().as_slice();
        let mut first_node = true;
        for child_node in nodes {
            if first_node {
                first_node = false;
            } else {
                write!(maud_template, "\n").unwrap();
            }
            handle_node(child_node.get(parser), parser, maud_template, indent + 1);
        }

        if !use_semicolon {
            write!(maud_template, "{}}}\n", spaces(indent)).unwrap();
        }
    }

    fn handle_node(
        node_opt: Option<&Node>,
        parser: &Parser,
        maud_template: &mut String,
        indent: usize,
    ) {
        match node_opt {
            None => {}
            Some(node) => match node {
                Node::Tag(tag) => handle_tag(tag, parser, maud_template, indent),
                Node::Comment(_) => {}
                Node::Raw(raw) => {
                    let text = raw.as_utf8_str();
                    let trimmed_text = text.trim();
                    if !trimmed_text.is_empty() {
                        write!(
                            maud_template,
                            "{}\"{}\"\n",
                            spaces(indent),
                            trimmed_text.replace("\"", "\\\"")
                        )
                        .unwrap();
                    }
                }
            },
        }
    }

    write!(maud_template, "html! {{\n").unwrap();
    for node_handle in dom.children() {
        handle_node(node_handle.get(parser), parser, &mut maud_template, 1);
    }
    write!(maud_template, "\n}}\n").unwrap();

    remove_empty_lines(&maud_template)
}
