use std::fs;
use std::net::TcpStream;
use std::io::{Read, Write, BufRead, BufReader};
use std::collections::HashMap;
use native_tls::TlsConnector;

trait Stream: Read + Write {}
impl Stream for TcpStream {}
impl<S: Read + Write> Stream for native_tls::TlsStream<S> {}

#[derive(Default)]
pub struct URLHandler {
    url: String,
    scheme: String,
    host: String,
    path: String,
    port: u16,
    view_source: bool,
    mediatype: String,
    data: String,
}

impl URLHandler {
    pub fn init(&mut self, url: String, view_source: bool) {
        self.view_source = view_source;
        
        if let Some((scheme, rest)) = url.split_once(':') {
            self.scheme = scheme.to_string();
            self.url = rest.to_string();
        }
        
        if self.scheme == "view-source" {
            self.view_source = true;
            if let Some((scheme, rest)) = self.url.split_once(":") {
                self.scheme = scheme.to_string();
                self.url = rest.to_string();
            }
        }
        
        if self.scheme == "data" {
            if self.url.contains(",") {
                if let Some((mediatype, data)) = self.url.split_once(",") {
                    self.mediatype = mediatype.to_string();
                    self.data = data.to_string();
                }
            } else {
                self.mediatype = "text/plain".to_string();
                self.data = self.url.clone();
            }
        } else {
            if let Some((_rest, url)) = self.url.split_once("//") {
                self.url = url.to_string();
            }
            let allowed_schemes = ["http", "https", "file"];
            assert!(
                allowed_schemes.contains(&self.scheme.as_str()),
                "Unsupported scheme"
            );
        }
        
        if self.scheme == "file" {
            self.path = self.url.clone();
        } else {
            if !self.url.contains("/") {
                self.url = self.url.clone() + "/";
            }
            if let Some((host, url)) = self.url.split_once("/") {
                self.host = host.to_string();
                self.url = url.to_string();
            }
            self.path = "/".to_string() + &self.url;
            
            if self.scheme == "http" {
                self.port = 80;
            } else if self.scheme == "https" {
                self.port = 443;
            }
            
            if self.host.contains(":") {
                if let Some((host, port)) = self.host.split_once(":") {
                    self.port = port.parse::<u16>().expect("Invalid port number");
                    self.host = host.to_string();
                }
            }
        }
    }

    fn request(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        const REDIRECT_LIMIT: i32 = 10;
        let mut redirects = 0;

        while redirects < REDIRECT_LIMIT {
            if self.scheme == "file" {
                return Ok(fs::read_to_string(&self.path)?);
            } else if self.scheme == "data" {
                return Ok(self.data.clone());
            } else {
                let stream = TcpStream::connect((&self.host[..], self.port))?;

                if self.scheme == "https" {
                    let connector = TlsConnector::new()?;
                    let tls_stream = connector.connect(&self.host, stream)?;
                    return self.handle_http_response(tls_stream, &mut redirects);
                } else {
                    return self.handle_http_response(stream, &mut redirects);
                }
            }
        }

        Err("Too many redirects".into())
    }

    fn handle_http_response<S: Read + Write>(
        &mut self,
        stream: S,
        redirects: &mut i32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        const REDIRECT_LIMIT: i32 = 10;

        loop {
            let mut stream = stream;
            
            let headers = vec![
                ("Host", self.host.as_str()),
                ("Connection", "keep-alive"),
                ("User-Agent", "Project P"),
                ("Accept-Encoding", "gzip"),
            ];

            let mut request = format!("GET {} HTTP/1.1\r\n", self.path);

            for (header, value) in &headers {
                request.push_str(&format!("{}: {}\r\n", header, value));
            }

            request.push_str("\r\n");

            stream.write_all(request.as_bytes())?;

            let mut reader = BufReader::new(stream);

            let mut statusline = String::new();
            reader.read_line(&mut statusline)?;
            let parts: Vec<&str> = statusline.split_whitespace().collect();
            let status = parts.get(1).ok_or("Invalid status line")?;

            let mut response_headers = HashMap::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line)?;
                if line == "\r\n" {
                    break;
                }
                if let Some((header, value)) = line.split_once(":") {
                    response_headers.insert(
                        header.trim().to_lowercase(),
                        value.trim().to_string(),
                    );
                }
            }

            if status.starts_with("3") {
                if let Some(location) = response_headers.get("location") {
                    if let Some(content_length) = response_headers.get("content-length") {
                        let length: usize = content_length.parse()?;
                        let mut buffer = vec![0u8; length];
                        reader.read_exact(&mut buffer)?;
                    } else {
                        let mut buffer = Vec::new();
                        reader.read_to_end(&mut buffer)?;
                    }

                    if location.starts_with("/") {
                        self.path = location.clone();
                    } else {
                        self.init(location.clone(), self.view_source);
                    }

                    *redirects += 1;
                    
                    if *redirects >= REDIRECT_LIMIT {
                        return Err("Too many redirects".into());
                    }
                    
                    return self.request();
                } else {
                    return Err(format!("Redirect without location header: {}", status).into());
                }
            }

            let content = if let Some(content_length) = response_headers.get("content-length") {
                let length: usize = content_length.parse()?;
                let mut buffer = vec![0u8; length];
                reader.read_exact(&mut buffer)?;
                String::from_utf8(buffer)?
            } else {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;
                String::from_utf8(buffer)?
            };

            assert!(
                !response_headers.contains_key("transfer-encoding"),
                "transfer-encoding not supported"
            );
            assert!(
                !response_headers.contains_key("content-encoding"),
                "content-encoding not supported"
            );

            return Ok(content);
        }
    }
}

pub fn show(body: &str, view_source: bool) {
    if view_source {
        print!("{}", body);
    } else {
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity_value = String::new();

        let mut entities = HashMap::new();
        entities.insert("gt".to_string(), ">".to_string());
        entities.insert("lt".to_string(), "<".to_string());

        for c in body.chars() {
            if c == '<' {
                in_tag = true;
            } else if c == '>' {
                in_tag = false;
            } else if c == '&' {
                in_entity = true;
            } else if c == ';' && in_entity {
                in_entity = false;
                if let Some(entity) = entities.get(&entity_value) {
                    print!("{}", entity);
                }
                entity_value.clear();
            } else if in_entity {
                entity_value.push(c);
            } else if !in_tag {
                print!("{}", c);
            }
        }
    }
}

pub fn load(mut url_handler: URLHandler) -> Result<(), Box<dyn std::error::Error>> {
    let body = url_handler.request()?;
    show(&body, url_handler.view_source);
    Ok(())
}