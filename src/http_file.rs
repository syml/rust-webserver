use std::io::prelude::*;
use http::*;
use std::fs::*;

pub struct FileSystem {
    path: String,
}
impl FileSystem {
    pub fn new(path: &str) -> FileSystem {
        FileSystem { path: path.to_string() }
    }
    pub fn serve(&mut self, uri: &str, resp: &mut Response) {
        let mut full_path = self.path.clone();
        full_path.push_str(uri);
        if let Ok(m) = metadata(&full_path) {
            if m.is_file() {
                if let Ok(ref mut f) = File::open(&full_path) {
                    resp.set_status(Status::ok());
                    resp.set_header("Content-Type", Self::get_mime(&full_path));
                    resp.set_length(m.len());
                    resp.send();
                    let buf_size = 10 * 1024 * 1024; // 10MB;
                    let mut buf: Vec<u8> = Vec::with_capacity(buf_size);
                    buf.resize(buf_size, 0);
                    while let Ok(n) = f.read(&mut buf) {
                        if n > 0 {
                            resp.send_data(&buf[0..n]);
                        } else {
                            break;
                        }
                    }
                    return;
                }
            }
        }
        println!("Not found: {}", full_path);
        resp.set_not_found().send();
    }

    fn get_mime(path: &str) -> &str {
        if let Some(idx) = path.rfind('.') {
            let ext = &path[idx + 1..];
            match ext {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "svg" => "image/svg+xml",
                "woff" | "woff2" => "application/x-font-woff",
                _ => "application/octet-stream",
            }
        } else {
            // Default to binary data.
            "application/octet-stream"
        }
    }
}
