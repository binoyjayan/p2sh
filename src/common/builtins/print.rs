use std::fmt;
use std::fmt::Write;
use std::rc::Rc;

use crate::common::object::*;

enum NumberFormat {
    Boolean,
    None,
    Octal,
    Hex,
    HexaDecimal,
}

enum SpecJustify {
    Default,
    Left,
    Right,
}

pub struct Collector(pub Vec<String>);

impl fmt::Write for Collector {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.push(s.to_string());
        Ok(())
    }
}

fn format_obj(
    collector: &mut Collector,
    padding: &str,
    justify: SpecJustify,
    width_str: &str,
    num_fmt: NumberFormat,
    obj: &Object,
) -> Result<(), String> {
    // Parse width
    let width: usize = if width_str.is_empty() {
        0
    } else {
        width_str
            .parse()
            .map_err(|_: std::num::ParseIntError| "Failed to parse width".to_string())?
    };

    // Format based on NumberFormat
    let formatted = match num_fmt {
        NumberFormat::Boolean => {
            if let Object::Number(num) = obj {
                format!("{:b}", *num as usize)
            } else {
                Err(String::from("Can't format non-number as binary"))?
            }
        }
        NumberFormat::Octal => {
            if let Object::Number(num) = obj {
                format!("{:o}", *num as usize)
            } else {
                Err(String::from("Can't format non-number as octal"))?
            }
        }
        NumberFormat::Hex => {
            if let Object::Number(num) = obj {
                format!("{:x}", *num as usize)
            } else {
                Err(String::from("Can't format non-number as hex"))?
            }
        }
        NumberFormat::HexaDecimal => {
            if let Object::Number(num) = obj {
                format!("{:X}", *num as usize)
            } else {
                Err(String::from("Can't format non-number as hex"))?
            }
        }
        NumberFormat::None => {
            format!("{}", obj)
        }
    };
    // Use a default padding of spaces
    let padding = if padding.is_empty() { " " } else { padding };
    let width_pad = width.saturating_sub(formatted.len());
    let padded: String = padding.repeat(width_pad);

    // Use default justification as right for number and left for everything else
    let justify = match justify {
        SpecJustify::Default => {
            if let Object::Number(_) = obj {
                SpecJustify::Right
            } else {
                SpecJustify::Left
            }
        }
        _ => justify,
    };

    // Handle justification and padding
    let justify = match justify {
        SpecJustify::Default => {
            if let Object::Number(_) = obj {
                SpecJustify::Right
            } else {
                SpecJustify::Left
            }
        }
        _ => justify,
    };

    // Handle justification and padding
    let formatted_output = match justify {
        SpecJustify::Left => format!("{}{}", formatted, padded),
        SpecJustify::Right => format!("{}{}", padded, formatted),
        _ => formatted,
    };

    write!(collector, "{}", formatted_output).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn format_buf(args: Vec<Rc<Object>>) -> Result<Collector, String> {
    if args.is_empty() {
        return Err(String::from("takes a minimum of one argument"));
    }
    let parts: Vec<char> = if let Object::Str(fmt) = &*args[0] {
        fmt.chars().collect()
    } else {
        return Err(String::from("Expected a string or format specifier"));
    };
    // Create a custom writer that collects the formatted output
    let mut collector = Collector(Vec::new());
    let mut idx_fmt = 0; // index to format specifier
    let mut idx_arg = 1; // index to args skipping the format specifier
    let mut in_spec = false; // inside '{}'
    let mut in_spec_format = false; // format specifier available after colon ':'
    let mut curr_spec_just = SpecJustify::Default;
    let mut curr_spec_width = String::new(); // store format specifier
    let mut curr_spec_padding = String::new(); // store padding specifier before '<' or '>'
    let mut curr_spec_idx = String::new(); // store index specifier {0}, {1}, etc
    let mut num_fmt: NumberFormat = NumberFormat::None;
    while idx_fmt < parts.len() {
        let curr = parts[idx_fmt];
        let next = if idx_fmt < parts.len() - 1 {
            parts[idx_fmt + 1]
        } else {
            '\0'
        };
        if curr == '{' {
            if next == '{' {
                write!(collector, "{{").map_err(|e| e.to_string())?;
                idx_fmt += 2; // skip next brace as well
            } else {
                in_spec = true;
                idx_fmt += 1;
            }
            continue;
        } else if curr == '}' {
            if next == '}' {
                write!(collector, "}}").map_err(|e| e.to_string())?;
                idx_fmt += 2; // skip next brace as well
                continue;
            }
            // Now print the arguments based on the specifier
            if idx_arg >= args.len() {
                return Err(String::from("positional arguments exceeded the count"));
            }

            // If index specifier is empty, use positional index 'idx_arg'
            // Otherwise, use the specified index into the arguments list
            if curr_spec_idx.is_empty() {
                // specifiers such as '{}', '{:10}', '{<5}', '{:0>5}' etc
                format_obj(
                    &mut collector,
                    &curr_spec_padding,
                    curr_spec_just,
                    &curr_spec_width,
                    num_fmt,
                    &args[idx_arg],
                )?;
                idx_arg += 1;
            } else {
                // specifiers such as '{0}', '{1}', '{0:10}', '{0<5}', '{1:0>5}' etc
                // 'args' also includes the format specifier. 'args.len()'
                // So, 'idx_print' should be the next element in args vector.
                let idx_print = curr_spec_idx.parse::<usize>().map_err(|e| e.to_string())? + 1;
                if idx_print >= args.len() {
                    return Err(String::from("positional argument index exceeded the count"));
                }
                format_obj(
                    &mut collector,
                    &curr_spec_padding,
                    curr_spec_just,
                    &curr_spec_width,
                    num_fmt,
                    &args[idx_print],
                )?;
            }

            in_spec = false;
            in_spec_format = false;
            curr_spec_just = SpecJustify::Default;
            curr_spec_width = String::new();
            curr_spec_padding = String::new();
            curr_spec_idx = String::new();
            num_fmt = NumberFormat::None;
            idx_fmt += 1;
            continue;
        }
        if in_spec {
            if curr == ':' {
                in_spec_format = true;
                idx_fmt += 1;
                continue;
            }
            if in_spec_format && (curr == '<' || curr == '>') {
                // the specifier is actually a padding one
                curr_spec_padding = curr_spec_width.clone();
                curr_spec_width = String::new();
                idx_fmt += 1;
                curr_spec_just = if curr == '<' {
                    SpecJustify::Left
                } else {
                    SpecJustify::Right
                };
                continue;
            }

            num_fmt = match curr {
                'b' => NumberFormat::Boolean,
                'o' => NumberFormat::Octal,
                'x' => NumberFormat::Hex,
                'X' => NumberFormat::HexaDecimal,
                _ => {
                    // If none of the numberic specifiers, it must be an int
                    if in_spec_format {
                        // Specifiers such as {:10}, {:0<5}, {:0>5}
                        curr_spec_width.push(curr);
                    } else {
                        // Specifiers such as {0}
                        curr_spec_idx.push(curr);
                    }
                    NumberFormat::None
                }
            };
        } else {
            // Treat characters outside specifier as printable ones
            write!(collector, "{}", curr).map_err(|e| e.to_string())?;
        }
        idx_fmt += 1;
    }
    Ok(collector)
}
