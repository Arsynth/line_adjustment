use std::{collections::LinkedList, iter::Peekable};

const SPACE_STR: &str = " ";
const NEWLINE_STR: &str = "\n";

/// Accepts string and adjusts it according the `line_width`.
/// Tries to fit words, separated by any whitespace to one line (limited by `line_width`).
/// Remaining words, that does not fit into one line will be moved to next line.
/// 
/// Any whitespace, that written manually will be replaced by `SPACE_STR` with variable length.
/// 
/// In the case when single word does not fit into line, this will be splitted into multiple lines.
/// Last line will be padded with leading `SPACE_STR` to fill whole line
pub fn transform(input: &str, line_width: u32) -> String {
    if input.chars().count() == 0 {
        return String::new();
    }

    let line_width = line_width as usize;

    let mut result = String::new();
    let tokens = input.split_whitespace();

    let mut need_newline = false;
    let mut peekable = tokens.peekable();

    while let Some(_) = peekable.peek() {
        let fit_result = fit_strs(&mut peekable, line_width);

        if need_newline {
            result += NEWLINE_STR;
        }

        if fit_result.list.len() != 0 {
            let gaps_info = gaps(fit_result.list.len(), fit_result.total_len, line_width);
            let n_gaps = fit_result.list.len() - 1;
            for (idx, token) in fit_result.list.iter().enumerate() {
                result += token;

                let next_idx = idx + 1;
                if next_idx < n_gaps {
                    result += &SPACE_STR.repeat(gaps_info.body_gaps_size);
                } else if next_idx == n_gaps || fit_result.list.len() == 1 {
                    result += &SPACE_STR.repeat(gaps_info.tail_gap_size);
                }
            }
        } else {
            // Case when even single word does not fit to required line length.
            // We should at least split it manually.
            let peeked = peekable
                .peek()
                .expect("Value is already peeked, but results in None");

            result += &split_manually(&peeked, line_width);

            // Force peekable to jump to the next element to prevent
            // stucking on large unconsumed word
            _ = peekable.next();
        }

        need_newline = true;
    }

    result
}

fn split_manually(unfitted_str: &str, line_width: usize) -> String {
    use std::cmp::min;

    let mut result = String::new();

    let str_len = unfitted_str.len();
    let mut elapsed = 0;

    let mut need_newline = false;

    while elapsed != str_len {
        let tail = &unfitted_str[elapsed..];

        // line_width is upper limit for characters counting
        let available_chars = tail.chars().take(line_width).count();
        let (available, chr) = tail
            .char_indices()
            .nth(available_chars - 1)
            .expect("String has reached end unexpectedly");

        let to_append_len = min(str_len - elapsed, available + chr.len_utf8());

        if need_newline {
            result += NEWLINE_STR;
        }

        result += &tail[..to_append_len];

        if available_chars < line_width {
            result += &SPACE_STR.repeat(line_width - available_chars);
        }

        elapsed += to_append_len;
        need_newline = true;
    }

    result
}

fn fit_strs<'a>(
    tokens: &mut Peekable<std::str::SplitWhitespace<'a>>,
    max_line_width: usize,
) -> FitResult<'a> {
    const ONE_SPACE: usize = 1;

    let mut list = LinkedList::<&str>::new();
    let mut total_len = 0;
    let mut chk_len = 0;

    while let Some(s) = tokens.next_if(|s| chk_len + s.chars().count() <= max_line_width) {
        let chars_count = s.chars().count();
        total_len += chars_count;

        // Assuming there will space before next word
        chk_len += chars_count + ONE_SPACE;

        list.push_back(s);
    }

    FitResult { list, total_len }
}

struct FitResult<'a> {
    list: LinkedList<&'a str>,
    total_len: usize,
}

fn gaps(n_tokens: usize, total_len: usize, line_width: usize) -> GapInfo {
    if n_tokens == 0 {
        return GapInfo {
            body_gaps_size: 0,
            tail_gap_size: 0,
        };
    } else if n_tokens == 1 {
        return GapInfo {
            body_gaps_size: 0,
            tail_gap_size: line_width - total_len,
        };
    }

    let n_gaps = n_tokens - 1;
    let free_space = line_width - total_len;
    let remainder = free_space % (n_gaps);

    let div = if n_gaps > 1 && remainder > 0 {
        n_gaps - 1
    } else {
        n_gaps
    };

    let max_gap = (free_space - remainder) / div;

    let last_gap = if remainder > 0 { remainder } else { max_gap };

    GapInfo {
        body_gaps_size: max_gap,
        tail_gap_size: last_gap,
    }
}

struct GapInfo {
    body_gaps_size: usize,
    tail_gap_size: usize,
}

#[cfg(test)]
mod tests {
    use super::transform;

    #[test]
    fn split_test() {
        let test_cases = [
            ("consectetur", 4, "cons\necte\ntur "),
            ("Привет", 12, "Привет      "),
            ("Поддержка кодировки utf-8 в коде", 8, "Поддержк\nа       \nкодировк\nи       \nutf-8  в\nкоде    "),
            ("Съешь ещё этих мягких французских булок, да выпей чаю", 12, "Съешь    ещё\nэтих  мягких\nфранцузских \nбулок,    да\nвыпей    чаю"),
            ("🤩 привет  💨 hello", 1, "🤩\nп\nр\nи\nв\nе\nт\n💨\nh\ne\nl\nl\no"),
            ("🤩 привет  💨 hello", 3, "🤩  \nпри\nвет\n💨  \nhel\nlo "),
        ];

        for &(input, line_width, expected) in &test_cases {
            println!("input: '{}'", input);
            assert_eq!(transform(input, line_width), expected);
        }
    }

    #[test]
    fn equal_length_lines() {
        let test_cases = [
            ("Бык тупогуб, тупогубенький бычок, у быка губа тупа.", 5),
            ("Вез корабль карамель, наскочил корабль на мель, матросы две недели карамель на мели ели.", 18),
            ("Вез корабль карамель, наскочил корабль на мель, матросы две недели карамель на мели ели.", 6),
            ("Вез корабль карамель, наскочил корабль на мель, матросы две недели карамель на мели ели.", 1),
            ("Тpидцaть тpи коpaбля лaвиpовaли, лaвиpовaли, лавировали, дa не \tвылaвиpовaли.", 4),
            ("У переп\tелa и перепелки\t\t\t пять  \t\tперепелят    .", 3),
        ];

        for (input, line_width) in test_cases {
            let result = transform(input, line_width);
            println!("input: '{}'", input);
            for line in result.lines() {
                assert_eq!(line.chars().count() as u32, line_width);
            }
        }
    }


}
