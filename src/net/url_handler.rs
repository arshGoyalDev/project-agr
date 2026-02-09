use std::fs;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::io::{Read, Write, BufRead, BufReader, self};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

use native_tls::TlsConnector;

use lazy_static::lazy_static;

use flate2::read::GzDecoder;

lazy_static! {
  static ref CACHE: Mutex<HashMap<String, CacheEntry>> = Mutex::new(HashMap::new());
}

#[derive(Clone)]
struct CacheEntry {
  content: String,
  timestamp: u64,
  max_age: Option<u64>,
}

#[derive(Default)]
pub struct URLHandler {
  url: String,
  scheme: String,
  host: String,
  path: String,
  port: u16,
  pub view_source: bool,
  mediatype: String,
  data: String,
}

impl URLHandler {
  pub fn init(&mut self, url: String, view_source: bool) {
    self.view_source = view_source;

    match self.parse_url(url.clone()) {
      Err(error) => {
        println!("Malformed URL: {url}. Loading about:blank instead");
        println!("{error:?}");
        self.scheme = String::from("about");
        self.data = String::from("Blank Page");
      }
      _ => ()
    }
  }

  fn parse_url(&mut self, url:String) -> Result<(), Error> {
    let (scheme, rest) = url.split_once(':')
      .ok_or(Error::new(ErrorKind::InvalidInput, "Malformed URL: missing ':'"))?;

    self.scheme = scheme.to_string();
    self.url = rest.to_string();

    if self.scheme == "view-source" {
      self.view_source = true;
      let (scheme, rest) = self.url.split_once(':')
        .ok_or(Error::new(ErrorKind::InvalidInput, "Malformed URL: missing ':'"))?;

      self.scheme = scheme.to_string();
      self.url = rest.to_string();
    }

    if self.scheme == "about" {
      self.data = String::from("Blank Page");
      return Ok(());
    }

    if self.scheme == "data" {
      if self.url.contains(",") {
        if let Some((mediatype, data)) = self.url.split_once(',') {
          self.mediatype = mediatype.to_string();
          self.data = data.to_string();
        }
      } else {
          self.mediatype = "text/plain".to_string();
          self.data = self.url.clone();
      }

      return Ok(());
    }

    let (_rest, url) = self.url.split_once("//")
      .ok_or(Error::new(ErrorKind::InvalidInput, "Malformed URL: missing '//' after scheme"))?;
    self.url = url.to_string();

    if !["http", "https", "file"].contains(&self.scheme.as_str()) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Malformed URL: Unsupported scheme: {}", self.scheme)
        ));
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
          self.port = port.parse::<u16>().map_err(|_| Error::new(ErrorKind::InvalidInput, format!("Malformed URL: Invalid Port: {port}")))?;
          self.host = host.to_string();
        }
      }

    }
    Ok(())
  }

  fn check_cache(&self, cache_key: &String) -> Option<String> {
    let cache = CACHE.lock().unwrap();

    if let Some(entry) = cache.get(cache_key) {
      let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

      if entry.max_age.is_none() {
        return Some(entry.content.clone());
      }

      if let Some(max_age) = entry.max_age {
        if current_time - entry.timestamp < max_age {
          return Some(entry.content.clone());
        }
      }
    }

    None
  }

  fn should_cache(&self, response_headers: &HashMap<String, String>, status: &str) -> (bool, Option<u64>) {
    if status != "200" {
      return (false, None);
    }

    let cache_control = response_headers
      .get("cache-control")
      .map(|s| s.as_str())
      .unwrap_or("");

    if cache_control.contains("no-store") {
      return (false, None);
    }

    if cache_control.contains("max-age") {
      for directive in cache_control.split(",") {
        let directive = directive.trim();
        if directive.starts_with("max-age=") {
          if let Some(value) = directive.split("=").nth(1) {
            if let Ok(max_age) = value.parse::<u64>() {
              return (true, Some(max_age));
            }
          }
        }
      }

      return (false, None);
    }

    let known_directives = ["no-store", "max-age", "public", "private"];
    let directives: Vec<&str> = cache_control
      .split(",")
      .map(|d| d.trim().split("=").next().unwrap_or(""))
      .filter(|d| !d.is_empty())
      .collect();

    for directive in directives {
      if !known_directives.contains(&directive) {
        return (false, None);
      }
    }

    (true, None)
  }

  pub fn request(&mut self) -> Result<String, Box<dyn std::error::Error>> {
    const REDIRECT_LIMIT: i32 = 10;
    let mut redirects = 0;

    while redirects < REDIRECT_LIMIT {
      if self.scheme == "file" {
        return Ok(fs::read_to_string(&self.path)?);
      } else if self.scheme == "data" {
        return Ok(self.data.clone());
      } else if self.scheme == "about" {
        return Ok(self.data.clone());
      } else {
        let cache_key = format!("{}://{}:{}{}", self.scheme, self.host, self.port, self.path);

        if let Some(cached_content) = self.check_cache(&cache_key) {
          println!("[Cache Hit] {}", cache_key);
          return Ok(cached_content);
        }

        println!("[Cache Miss] {}", cache_key);

        let stream = TcpStream::connect((&self.host[..], self.port))?;

        if self.scheme == "https" {
          let connector = TlsConnector::new()?;
          let tls_stream = connector.connect(&self.host, stream)?;
          return self.handle_http_response(tls_stream, &mut redirects, &cache_key);
        } else {
          return self.handle_http_response(stream, &mut redirects, &cache_key);
        }
      }
    }

    Err("Too many redirects".into())
  }

  fn handle_http_response<S: Read + Write>(
    &mut self,
    stream: S,
    redirects: &mut i32,
    cache_key: &str
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

          // clear the buffer before redirecting (good practice)
          if response_headers.get("transfer-encoding").is_some() {
            self.read_chunked(&mut reader)?;
          } else if let Some(content_length) = response_headers.get("content-length") {
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

      let mut raw_bytes = if response_headers.get("transfer-encoding").map(|v| v.as_str()) == Some("chunked") {
        self.read_chunked(&mut reader)?
      } else if let Some(content_length) = response_headers.get("content-length") {
        let length: usize = content_length.parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Content-Length"))?;
        let mut buffer = vec![0u8; length];
        reader.read_exact(&mut buffer)?;
        buffer
      } else {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        buffer
      };

      if response_headers.get("content-encoding").map(|v| v.as_str()) == Some("gzip") {
        println!("[Decompressing] gzip content");
        let mut decoder = GzDecoder::new(&raw_bytes[..]);
        let mut decompressed_bytes = Vec::new();
        decoder.read_to_end(&mut decompressed_bytes)?;
        raw_bytes = decompressed_bytes;
      }

      let content = String::from_utf8(raw_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;

      let (should_cache, max_age) = self.should_cache(&response_headers,status);
      if should_cache {
        let current_time = SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .as_secs();

        let entry = CacheEntry {
          content: content.clone(),
          timestamp: current_time,
          max_age,
        };

        let mut cache = CACHE.lock().unwrap();
        cache.insert(cache_key.to_string(), entry);

        if let Some(age) = max_age {
          println!("[Cached] {} (max-age: {}s)", cache_key, age);
        } else {
          println!("[Cached] {} (no expiry)", cache_key);
        }
      } else {
        println!("[Not Cached] {}", cache_key);
      }

      return Ok(content);
      }
  }

  fn read_chunked<R: BufRead>(&self ,reader: &mut R) -> io::Result<Vec<u8>> {
    let mut chunks = Vec::new();

    loop {
      let mut line = String::new();
      reader.read_line(&mut line)?;

      let chunk_size = usize::from_str_radix(line.trim(), 16)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid chunk size hex"))?;

      if chunk_size == 0 {
        loop {
          let mut trailer_line = String::new();
          reader.read_line(&mut trailer_line)?;
          if trailer_line == "\r\n" || trailer_line.is_empty() {
            break;
          }
        }
        break;
      }

      let mut chunk_data = vec![0u8; chunk_size];
      reader.read_exact(&mut chunk_data)?;
      chunks.extend(chunk_data);

      let mut footer = String::new();
      reader.read_line(&mut footer)?;
    }

    Ok(chunks)
  }
}

// pub fn show(body: &str, view_source: bool) {
//   if view_source {
//     print!("{}", body);
//   } else {
//     let mut in_tag = false;
//     let mut in_entity = false;
//     let mut entity_value = String::new();

//     let mut entities = HashMap::new();
//     entities.insert("gt".to_string(), ">".to_string());
//     entities.insert("lt".to_string(), "<".to_string());

//     for c in body.chars() {
//       if c == '<' {
//         in_tag = true;
//       } else if c == '>' {
//         in_tag = false;
//       } else if c == '&' {
//         in_entity = true;
//       } else if c == ';' && in_entity {
//         in_entity = false;
//         if let Some(entity) = entities.get(&entity_value) {
//           print!("{}", entity);
//         }
//         entity_value.clear();
//       } else if in_entity {
//         entity_value.push(c);
//       } else if !in_tag {
//         print!("{}", c);
//       }
//     }
//   }
// }

// pub fn load(mut url_handler: URLHandler) -> Result<(), Box<dyn std::error::Error>> {
//   let body = url_handler.request()?;
//   show(&body, url_handler.view_source);
//   Ok(())
// }
