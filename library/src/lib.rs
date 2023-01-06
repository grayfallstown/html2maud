
use std::fmt::Write;
use tl::*;

pub fn html2maud(html: &str) -> String {
    let mut maud_template = String::new();


    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    fn spaces(count: usize) -> String {
        return "    ".repeat(count).as_str().to_owned();
    }

    fn handle_tag(tag: &HTMLTag, parser: &Parser, maud_template: &mut String, indent: usize) {
        let tag_name = tag.name().as_utf8_str();
        write!(maud_template, "{}{}", spaces(indent), &tag_name).unwrap();
        let id = tag.attributes().id().map(|x| x.as_utf8_str());
        match &id {
            Option::Some(x) => {
                let escaped_id = if x.contains("-") {
                    format!("\"{}\"", &x)
                } else {
                    x.to_string()
                };
                write!(maud_template, " #{}", &escaped_id).unwrap();
            },
            Option::None => {},
        }

        match tag.attributes().class_iter() {
            None => {},
            Some(classes) => {
                for class in classes {
                    let escaped_class = if class.contains("-") {
                        format!("\"{}\"", &class)
                    } else {
                        class.to_owned()
                    };
                    
                    write!(maud_template, ".{}", &escaped_class).unwrap();
                }
            }
        }
        
        for (key, value_opt) in tag.attributes().iter() {
            if !(key.eq("id") || key.eq("class")) {
                write!(maud_template, " {}", key).unwrap();
                match value_opt {
                    None => {},
                    Some(value) => write!(maud_template, "=\"{}\"", value).unwrap(),
                }    
            }
        }

        write!(maud_template, " {{\n").unwrap();
        let children = tag.children();
        let nodes = children.top().as_slice();
        for child_node in nodes {
            handle_node(child_node.get(parser), parser, maud_template, indent + 1);
            write!(maud_template, "\n").unwrap();
        }
        write!(maud_template, "{}}}", spaces(indent)).unwrap();
    }

    fn handle_node(node_opt: Option<&Node>, parser: &Parser, maud_template: &mut String, indent: usize) {
        match node_opt {
            None => {},
            Some(node) => {
                match node {
                    Node::Tag(tag) => handle_tag(tag, parser, maud_template, indent),
                    Node::Comment(_) => {},
                    Node::Raw(raw) => {
                        let text = raw.as_utf8_str();
                        let trimmed_text = text.trim();
                        if !trimmed_text.is_empty() {
                            write!(maud_template, "{}\"{}\"", spaces(indent), trimmed_text.replace("\"","\\\"")).unwrap();
                        }
                    }
                }
            }
        }
    }

    write!(maud_template, "html! {{\n").unwrap();
    for node_handle in dom.children() {
        handle_node(node_handle.get(parser), parser, &mut maud_template, 1);
    }
    write!(maud_template, "\n}}\n").unwrap();
    

    maud_template

}
