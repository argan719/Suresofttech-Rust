// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use calamine::{open_workbook, DataType, Reader, Xlsx};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use rfd::FileDialog;


struct modifywordlist{
    before_list:Vec<String>,
    after_list:Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[tauri::command]
fn open_file() -> Result<String, String> {
    let file = FileDialog::new()
        .set_title("Select a file")
        .pick_file();

    // 파일이 선택되었는지 확인
    match file {
        Some(path) => {
            // 파일 경로를 문자열로 반환
            Ok(path.display().to_string())
        }
        None => {
            // 파일을 선택하지 않았을 경우 오류 메시지 반환
            Err("파일을 선택하지 않았습니다.".to_string())
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn convert_word(excel_add: &str, cs_add: &str) -> Result<String, String> {
    let mut excel: Xlsx<_> = open_workbook(excel_add).map_err(|_| "Can't find the file".to_string())?;

    let range = excel.worksheet_range("Sheet1").map_err(|_| "Can't find the sheet".to_string())?;

    let mut word_pairs = modifywordlist {
        before_list: Vec::new(),
        after_list: Vec::new(),
    };

    let mut before_col_index = None;
    let mut after_col_index = None;
    let mut start_row = 0;

    for (row_idx, row) in range.rows().take(200).enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            match cell.get_string() {
                Some("before") => before_col_index = Some(col_idx),
                Some("after") => after_col_index = Some(col_idx),
                _ => (),
            }
        }

        // 'before', 'after' 열을 찾으면 시작 행 설정
        if before_col_index.is_some() && after_col_index.is_some() {
            start_row = row_idx + 1; // 데이터를 그 다음 행부터 읽기
            break;
        }
    }

    // println!("{} , {}, {}\n", start_row, before_col_index.unwrap_or(0), after_col_index.unwrap_or(0));

    // 'before', 'after' 열을 찾지 못하면 에러 반환
    if before_col_index.is_none() || after_col_index.is_none() {
        return Err("Could not find 'before' or 'after' column".to_string());
    }

    let range1 = excel.worksheet_range("Sheet1").map_err(|_| "Can't find the sheet".to_string())?;

    for row in range1.rows().skip(start_row) {
        let before_cell = row.get(before_col_index.unwrap_or(0));
        let after_cell = row.get(after_col_index.unwrap_or(0));

        if before_cell.map_or(true, |cell| cell.is_empty()) || after_cell.map_or(true, |cell| cell.is_empty()) {
            break;
        }

        if let Some(before_value) = before_cell.and_then(|cell| cell.get_string()) {
            word_pairs.before_list.push(before_value.to_string());
        }

        if let Some(after_value) = after_cell.and_then(|cell| cell.get_string()) {
            word_pairs.after_list.push(after_value.to_string());
        }
    }

    // println!("{:?}", word_pairs);

    let cs_file = File::open(cs_add).map_err(|e| e.to_string())?;
    let cs_reader = BufReader::new(cs_file);

    let new_file_path = if let Some(new_file_path) = split_and_modify_file_name(cs_add) {
        println!("Success");
        new_file_path
    } else {
        println!("No extension found.");
        return Err("No extension found.".to_string());
    };

    let mut new_file = File::create(new_file_path).map_err(|e| e.to_string())?;

    for line in cs_reader.lines() {
        let mut modi_line = line.map_err(|e| e.to_string())?;

        for list_idx in 0..word_pairs.before_list.len() {
            modi_line = replace_in_string(&modi_line, &word_pairs.before_list[list_idx], &word_pairs.after_list[list_idx]);
        }

        new_file.write_all(modi_line.as_bytes()).map_err(|e| e.to_string())?;
        new_file.write_all(b"\n").map_err(|e| e.to_string())?;
    }

    Ok("Converting Success".to_string())
}

#[tauri::command(rename_all = "snake_case")]
fn renumbering_tc(cs_path: &str) -> Result<String, String> {
    let file = OpenOptions::new()
        .read(true)
        .open(cs_path)
        .map_err(|_| "File Open Error".to_string())?;

    let mut reader = BufReader::new(&file);
    let mut output = String::new();
    let mut test_case_number = 1;

    for line in reader.lines() {
        let line = line.map_err(|_| "File Read Error".to_string())?;
        if line.contains("TestCase_") {
            let new_number = format!("{:03}", test_case_number);
            let new_line = line.replace(
                &line[line.find("TestCase_").unwrap() + 9..line.find("TestCase_").unwrap() + 12],
                &new_number,
            );
            output.push_str(&new_line);
            output.push('\n');
            test_case_number += 1;
        } else {
            output.push_str(&line);
            output.push('\n');
        }
    }

    // 파일을 쓰기 모드로 다시 열고, 수정된 내용을 덮어씁니다.
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(cs_path)
        .map_err(|_| "File Write Error".to_string())?;

    file.write_all(output.as_bytes())
        .map_err(|_| "File Write Error".to_string())?;

    Ok("Conversion Complete".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, open_file, convert_word, renumbering_tc])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 파일명 분리 후 수정
fn split_and_modify_file_name(file_path: &str) -> Option<PathBuf> {
    let path = Path::new(file_path);

    match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => {
            let file_stem = stem.to_str().unwrap_or("");
            let file_ext = ext.to_str().unwrap_or("");
            
            // 새로운 파일명을 만듦: "파일명modified.확장자"
            let modified_file_name = format!("modified_{}.{}", file_stem, file_ext);
            
            // 새로운 PathBuf로 반환 (현재 파일의 부모 경로에 modified 파일명을 붙임)
            let mut new_path = PathBuf::from(path.parent().unwrap_or_else(|| Path::new("")));
            new_path.push(modified_file_name);

            Some(new_path)
        },
        _ => None,
    }
}

fn replace_in_string(original: &str, target_word: &str, replacement_word: &str) -> String {
    // `replace` 메서드를 사용하여 문자열에서 단어를 교체합니다.
    original.replace(target_word, replacement_word)
}