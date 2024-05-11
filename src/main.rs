use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::env::args;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead};
use tqdm::tqdm;

fn count_comment(file_path: &str, comment_regex: &Regex) -> Result<(i64, i64), Box<dyn Error>> {
    let file_name = file_path.split("/").last().unwrap();

    // count num lines
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let n_lines = reader.lines().count();

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut n_total_none_white_char = 0_i64;
    let mut n_comment_none_white_char = 0_i64;

    let mut pbar = tqdm(0..n_lines).smoothing(0.5).desc(Some(file_name));

    for line in reader.lines() {
        let l = line?;
        let d: Value = serde_json::from_str(&l)?;

        let content = d["content"].as_str().unwrap_or_default();
        // let content = d["result"].as_str().unwrap_or_default();
        let comment_matches: Vec<(usize, usize)> = comment_regex
            .find_iter(content)
            .map(|m| (m.start(), m.end()))
            .collect();
        let mark: Vec<i32> = (0..content.len())
            .map(|idx| {
                if comment_matches
                    .iter()
                    .any(|&(start, end)| start <= idx && idx < end)
                {
                    1
                } else {
                    0
                }
            })
            .collect();

        for (ch, is_comment) in content.chars().zip(mark) {
            if !ch.is_whitespace() {
                n_total_none_white_char += 1;
                if is_comment == 1 {
                    n_comment_none_white_char += 1;
                }
            }
        }
        pbar.next();
    }
    pbar.close()?;
    println!(
        "{}:\t{} / {}",
        file_name, n_comment_none_white_char, n_total_none_white_char
    );
    Ok((n_comment_none_white_char, n_total_none_white_char))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args().collect::<Vec<String>>();
    assert!(args.len() == 3);

    let lang = args[1].as_str();
    let lang_regex_mapping: HashMap<&str, &str> = HashMap::from([
        (
            "py",
            "(#.*)|(\'\'\'(?:.|\n)*?\'\'\')|(\"\"\"(?:.|\n)*?\"\"\")",
        ),
        ("java", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("js", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("go", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("php", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("rb", r"#.*|=begin(?:.|\n)*?=end"),
        ("rs", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("cpp", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("ts", "//.*|/\\*(?:.|\\n)*?\\*/"),
        ("cs", "//.*|/\\*(?:.|\\n)*?\\*/"),
    ]);

    let comment_regex = Regex::new(lang_regex_mapping.get(lang).unwrap())?;

    let data_path = &args[2];
    let file_paths = fs::read_dir(data_path)?;

    let mut n_comment_none_white_char = 0;
    let mut n_total_none_white_char = 0;

    for file_path in file_paths {
        let file_path = file_path?.path();
        let file_path = file_path.to_str().unwrap();
        let file_name = file_path.split("/").last().unwrap();
        if file_name.ends_with(".jsonl") {
            let (_n_comment_none_white_char, _n_total_none_white_char) =
                count_comment(file_path, &comment_regex)?;
            n_comment_none_white_char += _n_comment_none_white_char;
            n_total_none_white_char += _n_total_none_white_char;
        }
    }

    println!(
        "Total:\t{} / {}",
        n_comment_none_white_char, n_total_none_white_char
    );

    Ok(())
}
