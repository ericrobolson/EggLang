fn get_value<T>(value: Option<T>, msg: &str) -> Result<T, String> {
    match value {
        Some(v) => Ok(v),
        None => Err(msg.to_string()),
    }
}

pub fn serialize_packet(string: &str) -> String {
    const PACKET_FORMAT: &str = "Content-Length: {LEN}\r\n\r\n{CONTENT}";

    PACKET_FORMAT
        .replace("{LEN}", &string.len().to_string())
        .replace("{CONTENT}", string)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Packet {
    pub decoded: String,
    pub remaining: String,
}

pub fn read_packet(string: &str) -> Result<Packet, String> {
    let content_length_key = "Content-Length:";
    let content_length_index = get_value(
        string.find(content_length_key),
        "'Content-Length' not found",
    )?;

    let content_length_index = content_length_index + content_length_key.len();

    let return_key = "\r\n\r\n";
    let end_index = get_value(string.find(return_key), "'\\r\\n\\r\\n' not found")?;

    let content_length = &string[content_length_index..end_index];
    let content_length = match content_length.trim().parse::<usize>() {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Expected usize for content length, got '{}'",
                content_length.trim()
            ))
        }
    };

    let content_start = end_index + return_key.len();
    if string.len() < content_start + content_length {
        return Err(format!(
            "Message too short, expected {} characters",
            content_length
        ));
    }

    let content = &string[content_start..content_start + content_length];
    let remaining = &string[content_start + content_length..];

    Ok(Packet {
        decoded: content.trim().to_string(),
        remaining: remaining.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_packet_returns_value() {
        let string = r#"Hello World!"#;
        let result = serialize_packet(string);
        assert_eq!(result, "Content-Length: 12\r\n\r\nHello World!".to_string());
    }

    #[test]
    fn read_packet_returns_err_if_no_content_length() {
        let string = r#"
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/completion",
            "params": {
                ...
            }
        }
        "#;
        let result = read_packet(string);
        assert_eq!(result, Err("'Content-Length' not found".to_string()));
    }

    #[test]
    fn read_packet_returns_err_if_rnrn() {
        let string = r#"
Content-Length: 123 {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "textDocument/completion",
    "params": {
        ...
    }
}
        
        "#;
        let result = read_packet(string);
        assert_eq!(result, Err("'\r\n\r\n' not found".to_string()));
    }

    #[test]
    fn read_packet_returns_err_content_length_wrong_type() {
        let string = "
Content-Length: ABC\r\n\r\n
        {
        blahblah}
        ";
        let result = read_packet(string);
        assert_eq!(
            result,
            Err("Expected usize for content length, got 'ABC'".to_string())
        );
    }

    #[test]
    fn read_packet_returns_err_content_length_too_short() {
        let string = "
Content-Length: 123\r\n\r\n
";
        let result = read_packet(string);
        assert_eq!(
            result,
            Err("Message too short, expected 123 characters".to_string())
        );
    }

    #[test]
    fn read_packet_returns_value() {
        let string = "
Content-Length: 17\r\n\r\n
{
    blahblah
}The Rest of the Message";
        let result = read_packet(string);
        let expected = Packet {
            decoded: "{\n    blahblah\n}".to_string(),
            remaining: "The Rest of the Message".to_string(),
        };
        assert_eq!(result, Ok(expected));
    }
}
