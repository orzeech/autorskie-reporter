use std::{env, fs};
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::ptr::null;

use chrono::{Datelike, DateTime};
use walkdir::{DirEntry, WalkDir};

use model::GitLogElement;

mod model;

fn walk_repositories(repositories_folder: &String, output_directory: &String, year: i32, user: &String) {
    WalkDir::new(repositories_folder)
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .filter_entry(|e| is_git_directory(e))
        .filter_map(|v| v.ok())
        .map(|e| e.path().parent().unwrap().to_owned())
        .for_each(|x| generate_reports(get_git_log_elements(x.borrow(), year, user), output_directory, year, get_repository_url(x)));
}

fn get_repository_url(path: PathBuf) -> String {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path.to_str().unwrap())
        .output()
        .expect("failed to execute process");
    let success = output.status.success();
    if !success {
        return String::from("");
    }
    assert!(success);
    let cow = String::from_utf8_lossy(&output.stdout).to_string();
    String::from(&cow[..cow.len() - 5])
}

fn generate_reports(elements: Vec<GitLogElement>, output_directory: &String, year: i32, repository_url: String) {
    if elements.is_empty() {
        return;
    }
    let mut last_month: u32 = elements.get(0).unwrap().date.month();
    let mut file = open_file(output_directory, year, last_month);
    write_repository_header(&repository_url, &mut file);
    for el in elements {
        if last_month.ne(el.date.month().borrow()) {
            file = open_file(output_directory, year, el.date.month());
            write_repository_header(&repository_url, &mut file);
            last_month = el.date.month();
        }
        file.write(b"Date: ");
        file.write(el.date.to_string().as_bytes());
        file.write(b"\n");
        file.write(repository_url.as_ref());
        file.write(b"/");
        file.write(el.commit_id.as_ref());
        file.write(b"\n");
        file.write(b"Message: ");
        file.write(el.commit_message.as_ref());
        file.write(b"\n\n");
    }
}

fn write_repository_header(repository_url: &String, file: &mut File) {
    file.write(b"Repository: ");
    file.write(repository_url.as_ref());
    file.write(b"\n\n\n");
}

fn open_file(output_directory: &String, year: i32, m: u32) -> File {
    let mut name = String::from("raport_");
    name.push_str(m.to_string().as_str());
    name.push('_');
    name.push_str(year.to_string().as_str());
    name.push_str(".txt");
    let path = Path::new(output_directory).join(Path::new(name.as_str()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    file
}

fn get_git_log_elements(x: &PathBuf, year: i32, user: &String) -> Vec<GitLogElement> {
    let arg = get_git_log_arguments(year, user);
    let output = Command::new("git")
        .args(arg)
        .current_dir(x.to_str().unwrap())
        .output()
        .expect("failed to execute process");
    let success = output.status.success();
    let mut result: Vec<GitLogElement> = Vec::new();
    if !success {
        println!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
        return result;
    }
    assert!(success);
    let cow = String::from_utf8_lossy(&output.stdout);
    let mut lines = cow.lines();
    let mut line = lines.next();
    let mut counter = 0;
    let mut commit_id = "";
    let mut date = "";
    let mut commit_message = String::from("");
    let mut first_line = true;
    while line.is_some() {
        let line_str = line.unwrap();
        if line_str.starts_with("commit") {
            counter = 0;
            if !first_line {
                let date_time = DateTime::parse_from_rfc2822(&date[8..]);
                if date_time.is_ok() {
                    result.push(GitLogElement {
                        commit_id: commit_id[7..].parse().unwrap(),
                        date: date_time.unwrap(),
                        commit_message: commit_message[4..].parse().unwrap(),
                    });
                }
                commit_message = String::from("");
            } else { first_line = false }
        }
        match counter {
            0 => commit_id = line_str,
            1 => {}
            2 => date = line_str,
            _ => commit_message.push_str(line_str)
        };
        counter += 1;
        line = lines.next();
    }
    result
}

fn get_git_log_arguments(year: i32, user: &String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(String::from("log"));
    result.push(String::from("--since"));
    let mut since_arg = String::from(&*year.to_string());
    since_arg.push_str("-01-01");
    result.push(since_arg);

    result.push(String::from("--until"));
    let mut until_arg = String::from(year.to_string().as_str());
    until_arg.push_str("-12-31");
    result.push(until_arg);
    result.push(String::from("--author"));
    result.push(String::from(user));
    result.push(String::from("--date"));
    result.push(String::from("rfc"));
    result
}

fn is_git_directory(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.eq(".git"))
        .unwrap_or(false)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("Usage: {} path_to_git_repos output_dir year user", args[0]);
        exit(1)
    }
    let repositories_directory = &args[1];
    let output_directory = &args[2];
    let year_string = &args[3];
    let year: i32 = year_string.parse().unwrap();
    let user = &args[4];
    walk_repositories(repositories_directory, output_directory, year, user);
}
