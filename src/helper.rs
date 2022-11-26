use crate::*;

#[derive(Debug)]
pub struct Helper {
    glue: String,
    args: Vec<(usize, usize)>,
}

impl Helper {
    pub fn parse(value: &str) -> (usize, Self) {
        let mut args: Vec<(usize, usize, usize)> = Vec::new();

        // Extract arguments
        let mut to_scan = value;
        while let Some(idx_start) = get_idx_after_strict(to_scan, "[") {
            let after_part = &to_scan[idx_start..];
            if let Some(idx_end) = get_idx_before_strict(after_part, "]") {
                let idx_end = idx_start + idx_end;
                let arg = to_scan[idx_start..idx_end].to_string();

                // Make sure all characters are digits
                if arg.chars().all(|c| c.is_ascii_digit()) {
                    let arg = arg.parse::<usize>().unwrap();
                    args.push((arg, idx_start + (value.len() - to_scan.len()), idx_end + (value.len() - to_scan.len())));
                }

                to_scan = &to_scan[idx_end..];
            } else {
                break;
            }
        }

        // Make sure all args from 0 to the last one are present
        let mut max = 0;
        for (arg, _, _) in &args {
            if *arg > max {
                max = *arg;
            }
        }
        for i in 0..=max {
            if !args.iter().any(|(arg, _, _)| *arg == i) {
                panic!("Missing argument {} in helper", i);
            }
        }

        // Remove arguments from value
        let mut value = value.to_string();
        let mut removed_offset = 0;
        let mut args2 = Vec::new();
        for (arg, idx_start, idx_end) in args {
            value.replace_range(idx_start - removed_offset - 1..idx_end - removed_offset + 1, "");
            args2.push((arg, idx_start - 1 - removed_offset));
            removed_offset += (idx_end - idx_start) + 2;
        }

        (max+1, Helper { glue: value, args: args2 })
    }

    pub fn to_code(&self, values: Vec<String>) -> String {
        let mut code = self.glue.clone();
        for (arg, idx) in self.args.iter().rev() {
            code.insert_str(*idx, &values[*arg]);
        }
        code
    }
}

#[cfg(test)]
#[test]
fn test_helper() {
    let helper = Helper::parse("[0].to_string()").1;
    let code = helper.to_code(vec![String::from("15")]);
    assert_eq!(code, "15.to_string()");

    let helper = Helper::parse("[0] + [1] - [2] + [1]").1;
    let code = helper.to_code(vec![String::from("15"), String::from("10"), String::from("5")]);
    assert_eq!(code, "15 + 10 - 5 + 10");
}
