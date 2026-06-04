use eframe::egui;
use egui::{
    Color32, FontId, TextStyle,
    text::{CCursor, CCursorRange, LayoutJob, TextFormat},
};

pub fn show(ui: &mut egui::Ui, title: &str, text: &mut String, dirty: &mut bool) {
    ui.horizontal(|ui| {
        ui.heading(title);
    });
    ui.separator();

    if title == "No file open" {
        ui.centered_and_justified(|ui| {
            ui.label("Open a C/C++ file from the project tree.");
        });
        return;
    }

    let before = text.clone();
    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut job = highlight_cpp(ui, text);
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    // Enable scrolling
    let available_size = ui.available_size();
    let desired_rows = text.lines().count().max(32);
    let mut output = None;
    egui::ScrollArea::both()
        .id_salt("code_editor_scroll")
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, _viewport| {
            output = Some(
                egui::TextEdit::multiline(text)
                    .id_salt("lite_dev_cpp_code_editor")
                    .font(TextStyle::Monospace)
                    .desired_width(available_size.x.max(600.0))
                    .desired_rows(desired_rows)
                    .lock_focus(true)
                    .code_editor()
                    .layouter(&mut layouter)
                    .show(ui),
            );
        });

    let Some(mut output) = output else {
        return;
    };

    if output.response.changed() {
        if let Some(cursor_index) = output
            .cursor_range
            .and_then(|range| range.single())
            .map(|cursor| cursor.ccursor.index)
        {
            if let Some(next_cursor_index) = apply_edit_assists(&before, text, cursor_index) {
                output
                    .state
                    .cursor
                    .set_char_range(Some(CCursorRange::one(CCursor::new(next_cursor_index))));
                output.state.store(ui.ctx(), output.response.id);
            }
        }
        *dirty = true;
    }
}

// Right parenthesis, right quotation marks auto-complete
fn apply_edit_assists(before: &str, after: &mut String, cursor_index: usize) -> Option<usize> {
    let inserted = inserted_text(before, after)?;
    match inserted.as_str() {
        "(" => insert_pair(after, cursor_index, ")"),
        "[" => insert_pair(after, cursor_index, "]"),
        "{" => insert_pair(after, cursor_index, "}"),
        "\"" => insert_pair(after, cursor_index, "\""),
        "'" => insert_pair(after, cursor_index, "'"),
        "\t" => {
            replace_range(after, cursor_index.saturating_sub(1), cursor_index, "    ");
            Some(cursor_index + 3)
        }
        "\n" => apply_newline_indent(after, cursor_index),
        _ => None,
    }
}

fn inserted_text(before: &str, after: &str) -> Option<String> {
    let before_chars: Vec<char> = before.chars().collect();
    let after_chars: Vec<char> = after.chars().collect();
    if after_chars.len() <= before_chars.len() {
        return None;
    }

    let mut prefix = 0;
    while prefix < before_chars.len()
        && prefix < after_chars.len()
        && before_chars[prefix] == after_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < before_chars.len().saturating_sub(prefix)
        && suffix < after_chars.len().saturating_sub(prefix)
        && before_chars[before_chars.len() - 1 - suffix]
            == after_chars[after_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    Some(
        after_chars[prefix..after_chars.len() - suffix]
            .iter()
            .collect(),
    )
}

fn insert_pair(text: &mut String, cursor_index: usize, closing: &str) -> Option<usize> {
    let byte_index = byte_index_for_char(text, cursor_index)?;
    text.insert_str(byte_index, closing);
    Some(cursor_index)
}

fn apply_newline_indent(text: &mut String, cursor_index: usize) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    if cursor_index == 0 {
        return None;
    }

    let previous_line_start = chars[..cursor_index - 1]
        .iter()
        .rposition(|ch| *ch == '\n')
        .map_or(0, |index| index + 1);
    let base_indent: String = chars[previous_line_start..cursor_index - 1]
        .iter()
        .take_while(|ch| **ch == ' ' || **ch == '\t')
        .collect();
    let before_newline = chars[..cursor_index - 1]
        .iter()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .copied();
    let after_cursor = chars.get(cursor_index).copied();
    let extra_indent = if before_newline == Some('{') {
        "    "
    } else {
        ""
    };

    let insertion = if before_newline == Some('{') && after_cursor == Some('}') {
        format!("{base_indent}{extra_indent}\n{base_indent}")
    } else {
        format!("{base_indent}{extra_indent}")
    };

    if insertion.is_empty() {
        return None;
    }

    let byte_index = byte_index_for_char(text, cursor_index)?;
    text.insert_str(byte_index, &insertion);
    Some(cursor_index + base_indent.chars().count() + extra_indent.chars().count())
}

fn replace_range(text: &mut String, start_char: usize, end_char: usize, replacement: &str) {
    if let (Some(start), Some(end)) = (
        byte_index_for_char(text, start_char),
        byte_index_for_char(text, end_char),
    ) {
        text.replace_range(start..end, replacement);
    }
}

fn byte_index_for_char(text: &str, char_index: usize) -> Option<usize> {
    if char_index == text.chars().count() {
        return Some(text.len());
    }
    text.char_indices().nth(char_index).map(|(index, _)| index)
}

// Code highlight
fn highlight_cpp(ui: &egui::Ui, code: &str) -> LayoutJob {
    let font_id = FontId::monospace(15.0);
    let default = TextFormat::simple(font_id.clone(), ui.visuals().text_color());
    let keyword = TextFormat::simple(font_id.clone(), Color32::from_rgb(46, 92, 190));
    let type_name = TextFormat::simple(font_id.clone(), Color32::from_rgb(128, 78, 160));
    let number = TextFormat::simple(font_id.clone(), Color32::from_rgb(170, 95, 0));
    let string = TextFormat::simple(font_id.clone(), Color32::from_rgb(150, 72, 40));
    let comment = TextFormat::simple(font_id.clone(), Color32::from_rgb(92, 130, 86));
    let preprocessor = TextFormat::simple(font_id.clone(), Color32::from_rgb(156, 85, 28));
    let operator = TextFormat::simple(font_id, Color32::from_rgb(85, 105, 125));

    let mut job = LayoutJob::default();
    let bytes = code.as_bytes();
    let mut index = 0;
    let mut line_start = true;

    while index < bytes.len() {
        let rest = &code[index..];

        if line_start && rest.trim_start().starts_with('#') {
            let leading = rest.len() - rest.trim_start().len();
            if leading > 0 {
                append(&mut job, &rest[..leading], &default);
                index += leading;
            }
            let len = code[index..].find('\n').unwrap_or(code[index..].len());
            append(&mut job, &code[index..index + len], &preprocessor);
            index += len;
            line_start = false;
            continue;
        }

        if rest.starts_with("//") {
            let len = rest.find('\n').unwrap_or(rest.len());
            append(&mut job, &rest[..len], &comment);
            index += len;
            line_start = false;
            continue;
        }

        if rest.starts_with("/*") {
            let len = rest.find("*/").map_or(rest.len(), |end| end + 2);
            append(&mut job, &rest[..len], &comment);
            line_start = rest[..len].ends_with('\n');
            index += len;
            continue;
        }

        if rest.starts_with('"') || rest.starts_with('\'') {
            let quote = bytes[index];
            let len = quoted_len(&bytes[index..], quote);
            append(&mut job, &code[index..index + len], &string);
            line_start = false;
            index += len;
            continue;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };

        if ch.is_ascii_alphabetic() || ch == '_' {
            let len = rest
                .char_indices()
                .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
                .map_or(rest.len(), |(index, _)| index);
            let word = &rest[..len];
            let format = if is_keyword(word) {
                &keyword
            } else if is_builtin_type(word) {
                &type_name
            } else {
                &default
            };
            append(&mut job, word, format);
            index += len;
            line_start = false;
            continue;
        }

        if ch.is_ascii_digit() {
            let len = rest
                .char_indices()
                .find(|(_, ch)| {
                    !ch.is_ascii_alphanumeric() && !matches!(*ch, '.' | '_' | '\'' | '+' | '-')
                })
                .map_or(rest.len(), |(index, _)| index);
            append(&mut job, &rest[..len], &number);
            index += len;
            line_start = false;
            continue;
        }

        let len = ch.len_utf8();
        let format = if "{}[]();,+-*/%=!<>&|^~?:.".contains(ch) {
            &operator
        } else {
            &default
        };
        append(&mut job, &rest[..len], format);
        line_start = ch == '\n' || (line_start && ch.is_whitespace());
        index += len;
    }

    job
}

fn append(job: &mut LayoutJob, text: &str, format: &TextFormat) {
    job.append(text, 0.0, format.clone());
}

fn quoted_len(bytes: &[u8], quote: u8) -> usize {
    let mut index = 1;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\n' {
            return index;
        }
        if byte == quote && !escaped {
            return index + 1;
        }
        escaped = byte == b'\\' && !escaped;
        if byte != b'\\' {
            escaped = false;
        }
        index += 1;
    }
    bytes.len()
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "alignas"
            | "alignof"
            | "asm"
            | "auto"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "concept"
            | "const"
            | "constexpr"
            | "continue"
            | "decltype"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "explicit"
            | "export"
            | "extern"
            | "for"
            | "friend"
            | "goto"
            | "if"
            | "inline"
            | "mutable"
            | "namespace"
            | "new"
            | "noexcept"
            | "operator"
            | "private"
            | "protected"
            | "public"
            | "register"
            | "requires"
            | "return"
            | "sizeof"
            | "static"
            | "static_assert"
            | "struct"
            | "switch"
            | "template"
            | "this"
            | "throw"
            | "try"
            | "typedef"
            | "typename"
            | "using"
            | "virtual"
            | "volatile"
            | "while"
    )
}

fn is_builtin_type(word: &str) -> bool {
    matches!(
        word,
        "bool"
            | "char"
            | "char16_t"
            | "char32_t"
            | "double"
            | "float"
            | "int"
            | "long"
            | "short"
            | "signed"
            | "unsigned"
            | "void"
            | "wchar_t"
            | "size_t"
            | "string"
            | "vector"
            | "map"
            | "set"
            | "queue"
            | "stack"
            | "deque"
            | "pair"
            | "tuple"
            | "priority_queue"
    )
}
