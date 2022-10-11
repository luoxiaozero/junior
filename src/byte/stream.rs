use crate::{JuriCustomError, Request};
use regex::Regex;
use std::collections::HashMap;

fn handle_request_line_bytes(line_bytes: Vec<u8>) -> (String, String, String) {
    let line = String::from_utf8(line_bytes).unwrap();
    let re = Regex::new(r"^(.*?) (.*?) (.*?)$").unwrap();
    let caps = re.captures(&line).unwrap();
    let method = caps
        .get(1)
        .map_or("".to_string(), |m| m.as_str().to_string());
    let full_path = caps
        .get(2)
        .map_or("".to_string(), |m| m.as_str().to_string());
    let version = caps
        .get(3)
        .map_or("".to_string(), |m| m.as_str().to_string());
    (method, full_path, version)
}

fn handle_header_bytes(header_bytes: Vec<u8>) -> (String, String) {
    let header = String::from_utf8(header_bytes).unwrap();
    let re = Regex::new(r"^(.*?):(.*?)$").unwrap();
    let caps = re.captures(&header).unwrap();
    let key = caps
        .get(1)
        .map_or("".to_string(), |m| m.as_str().trim().to_string());
    let value = caps
        .get(2)
        .map_or("".to_string(), |m| m.as_str().trim().to_string());
    (key, value)
}

pub struct JuriStream {
    request_line: Option<(String, String, String)>,
    header_map: HashMap<String, String>,
    body_bytes: Vec<u8>,
    multipart_form_data: Option<MultipartFormData>,
}

impl JuriStream {
    pub fn new() -> Self {
        JuriStream {
            request_line: None,
            header_map: HashMap::new(),
            body_bytes: vec![],
            multipart_form_data: None,
        }
    }

    pub fn handle_request_header_bytes(&mut self, header_bytes: Vec<u8>) {
        if let None = self.request_line {
            self.request_line = Some(handle_request_line_bytes(header_bytes));
        } else {
            let (key, value) = handle_header_bytes(header_bytes);
            self.header_map.insert(key, value);
        }
    }

    pub fn handle_request_body_bytes(&mut self, body_bytes: &mut Vec<u8>) {
        if let Some(multipart_form_data) = self.multipart_form_data.as_mut() {
            multipart_form_data.handle_bytes(body_bytes);
        } else {
            self.body_bytes.append(body_bytes);
        }
    }

    pub fn header_end(&mut self) {
        self.is_multipart_form_data();
    }

    pub fn get_request(self) -> Result<Request, JuriCustomError> {
        let mut request = Request::new();
        let request_line = self.request_line.map_or(
            Err(JuriCustomError {
                code: 400,
                reason: "请求方法错误".to_string(),
            }),
            |v| Ok(v),
        )?;
        request.method = request_line.0;
        request.set_full_path(request_line.1);
        request.version = request_line.2;

        request.header_map = self.header_map;

        request.body_bytes = self.body_bytes;

        Ok(request)
    }
}

impl JuriStream {
    pub fn is_multipart_form_data(&mut self) -> bool {
        if let Some(content_type) = self.header_map.get("Content-Type") {
            let re = Regex::new(r"^multipart/form-data; boundary=(.*?)$").unwrap();
            if let Some(caps) = re.captures(&content_type) {
                if let Some(boundary) = caps.get(1).map(|m| m.as_str()) {
                    self.multipart_form_data = Some(MultipartFormData {
                        boundary: boundary.to_string(),
                        form_data_vec: vec![],
                        body_bytes: vec![],
                        temp_form_data: None,
                    });
                    return true;
                }
            }
        }
        false
    }
}

struct MultipartFormData {
    boundary: String,
    form_data_vec: Vec<FormData>,
    body_bytes: Vec<u8>,
    temp_form_data: Option<FormData>,
}

impl MultipartFormData {
    pub fn handle_bytes(&mut self, body_bytes: &mut Vec<u8>) {
        let boundary_start_vec = format!("--{}", self.boundary).as_bytes().to_vec();
        let boundary_end_vec = format!("--{}--", self.boundary).as_bytes().to_vec();

        let mut flag_n = false; // 10
        let mut flag_r = false; // 13
        let mut point_index: usize = 0;
        println!("----");
        for (index, byte) in body_bytes.iter().enumerate() {
            if flag_r {
                if *byte == 10 {
                    flag_n = true;
                } else {
                    flag_r = false;
                }
            }

            if *byte == 13 {
                flag_r = true;
            }
            
            if flag_n && flag_r {
                let bytes = body_bytes[point_index..(index - 1)].to_vec();
                println!("n r {:#?}", String::from_utf8(bytes.clone()));
                if let Some(temp_form_data) = self.temp_form_data.as_mut() {
                    if temp_form_data.cache_file_name.is_empty() {

                    } else {

                    }
                }
                if is_vec_equals(&boundary_start_vec, &bytes) {
                    println!("boundary_end_vec 1");
                    self.temp_form_data = Some(FormData {
                        name: "".to_string(),
                        file_name: None,
                        content_type: None,
                        cache_file_name: "".to_owned(),
                    });
                    point_index = index + 1;
                } else if is_vec_equals(&boundary_end_vec, &bytes) {
                    println!("boundary_end_vec");
                    break;
                }
            }
        }
    }
}

struct FormData {
    name: String,
    file_name: Option<String>,
    content_type: Option<String>,

    cache_file_name: String,
}

fn is_vec_equals<T: std::cmp::PartialEq>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }

    for i in 0..vec1.len() {
        if vec1[i] != vec2[i] {
            return false;
        }
    }

    return true;
}
