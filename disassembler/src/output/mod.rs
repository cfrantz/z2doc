use structopt::clap::arg_enum;

arg_enum! {
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Html,
}
}

pub fn comment(fmt: Format, comment: &str) -> String {
    match fmt {
        Format::Text => format!("; {}", comment),
        Format::Html => format!("<span class=\"asm-comment\">; {}</span>", comment),
    }
}

pub fn directive(fmt: Format, d: &str) -> String {
    match fmt {
        Format::Text => d.to_string(),
        Format::Html => format!("<span class=\"asm-directive\">{}</span>", d),
    }
}

pub fn commentblock(fmt: Format, block: &str) -> Vec<String> {
    if !block.is_empty() {
        block
            .split('\n')
            .map(|s| comment(fmt, s))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    }
}

pub fn symbolref(fmt: Format, symbol: &str) -> String {
    match fmt {
        Format::Text => symbol.to_string(),
        Format::Html => format!(
            "<a href=\"#{}\" class=\"asm-symbol\">{}</a>",
            symbol, symbol
        ),
    }
}

pub fn label(fmt: Format, symbol: &str) -> String {
    match fmt {
        Format::Text => format!("{}:", symbol),
        Format::Html => format!(
            "<span id=\"{}\" class=\"asm-label\">{}</span>:",
            symbol, symbol
        ),
    }
}

fn constant(fmt: Format, value: &str) -> String {
    match fmt {
        Format::Text => value.to_string(),
        Format::Html => format!("<span class=\"asm-const\">{}</span>", value),
    }
}

fn address(fmt: Format, value: &str) -> String {
    match fmt {
        Format::Text => value.to_string(),
        Format::Html => format!("<span class=\"asm-addr\">{}</span>", value),
    }
}

pub fn equate(fmt: Format, symbol: &str, value: &str) -> String {
    let value = constant(fmt, value);
    match fmt {
        Format::Text => format!("{} = {}", symbol, value),
        Format::Html => format!(
            "<span id=\"{}\" class=\"asm-label\">{}</span> = {}",
            symbol, symbol, value
        ),
    }
}

pub fn instruction(
    fmt: Format,
    mnemonic: &str,
    operand: &str,
    symbol: Option<&str>,
    addr: u16,
    hex: &str,
    cmt: &str,
) -> String {
    const LEN: usize = 50;
    let (operand, n) = if let Some(s) = symbol {
        (symbolref(fmt, s), s.len())
    } else {
        if mnemonic == ".byte @" || mnemonic.contains('#') {
            (constant(fmt, operand), operand.len())
        } else {
            (address(fmt, operand), operand.len())
        }
    };
    let mut i = match fmt {
        Format::Text => {
            let mut i = String::from("    ");
            i.push_str(&mnemonic.replace("@", &operand));
            for _ in i.len()..LEN {
                i.push(' ');
            }
            i
        }
        Format::Html => {
            let mut i = String::from("    <span class=\"asm-code\">");
            let m = if mnemonic.contains('@') { 1 } else { 0 };
            i.push_str(&mnemonic.replace("@", &operand));
            i.push_str("</span>");
            for _ in mnemonic.len() + n - m..LEN {
                i.push(' ');
            }
            i
        }
    };
    i.push_str(&comment(fmt, &format!("{:04X} {:<8} ; {}", addr, hex, cmt)));
    i
}

pub fn document(fmt: Format, title: &str, style: &str, lines: Vec<String>) -> Vec<String> {
    match fmt {
        Format::Text => lines,
        Format::Html => {
            let mut doc = Vec::new();
            doc.push("<html>".to_string());
            doc.push("<head>".to_string());
            doc.push(format!("<title>{}</title>", title));
            doc.push("<style>".to_string());
            doc.push(style.to_string());
            doc.push("</style>".to_string());
            doc.push("</head>".to_string());
            doc.push("<body>".to_string());
            doc.push("<table>".to_string());
            for line in lines {
                doc.push("<tr>".to_string());
                doc.push(format!("<td>{}</td>", line));
                doc.push("</tr>".to_string());
            }
            doc.push("</table>".to_string());
            doc.push("</body>".to_string());
            doc.push("</html>".to_string());
            doc
        }
    }
}
