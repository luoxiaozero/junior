use super::form_data::MultipartFormData;
use crate::{request::HTTPMethod, Request};
use regex::Regex;
use std::collections::HashMap;

fn get_request_line(line: String) -> (String, String, String) {
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

pub fn get_header(header: String) -> (String, String) {
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

pub struct ReadStream {
    request_line: Option<(String, String, String)>,
    pub header_map: HashMap<String, String>,
    body_bytes: Vec<u8>,
    multipart_form_data: Option<MultipartFormData>,
}

impl ReadStream {
    pub fn new() -> Self {
        ReadStream {
            request_line: None,
            header_map: HashMap::new(),
            body_bytes: vec![],
            multipart_form_data: None,
        }
    }

    pub fn write_header(&mut self, header_bytes: Vec<u8>) {
        let header = String::from_utf8(header_bytes).unwrap();
        if let None = self.request_line {
            self.request_line = Some(get_request_line(header));
        } else {
            let (key, value) = get_header(header);
            self.header_map.insert(key, value);
        }
    }

    pub async fn write_body(&mut self, body_bytes: &mut Vec<u8>) {
        if let Some(multipart_form_data) = self.multipart_form_data.as_mut() {
            multipart_form_data.handle_bytes(body_bytes).await;
        } else {
            self.body_bytes.append(body_bytes);
        }
    }

    pub fn header_end(&mut self) {
        self.is_multipart_form_data();
    }

    pub fn get_request(self) -> Result<Request, crate::Error> {
        let mut request = Request::new();
        let request_line = self.request_line.map_or(
            Err(crate::Error {
                code: 400,
                reason: "请求方法错误".to_string(),
            }),
            |v| Ok(v),
        )?;
        request.method = HTTPMethod::from(request_line.0)?;
        request.set_full_path(request_line.1);
        request.protocol_and_version = request_line.2;

        request.header_map = self.header_map;

        request.body_bytes = self.body_bytes;
        if let Some(multipart_form_data) = self.multipart_form_data {
            request.multipart_form_data = multipart_form_data.form_data_vec;
        }

        Ok(request)
    }
}

impl ReadStream {
    pub fn is_multipart_form_data(&mut self) -> bool {
        if let Some(content_type) = self.header_map.get("Content-Type") {
            let re = Regex::new(r"^multipart/form-data; boundary=(.*?)$").unwrap();
            if let Some(caps) = re.captures(&content_type) {
                if let Some(boundary) = caps.get(1).map(|m| m.as_str()) {
                    self.multipart_form_data = Some(MultipartFormData {
                        boundary: boundary.to_string(),
                        form_data_vec: vec![],
                        temp_form_data: None,
                    });
                    return true;
                }
            }
        }
        false
    }
}
