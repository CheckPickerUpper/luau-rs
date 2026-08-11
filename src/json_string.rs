/// Appends one JSON string body without adding surrounding quotation marks.
pub fn append_json_string(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                append_hex_escape(output, character);
            }
            character => output.push(character),
        }
    }
}

fn append_hex_escape(output: &mut String, character: char) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let code = u32::from(character);
    output.push_str("\\u");
    for shift in [12, 8, 4, 0] {
        output.push(char::from(HEX_DIGITS[((code >> shift) & 0x0f) as usize]));
    }
}
