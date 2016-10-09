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
                resp.set_length(m.len());
                if let Ok(ref mut f) = File::open(&full_path) {
                    resp.set_status(Status::ok())
                        .set_header("Content-Type", "text/html")
                        .send();
                    let buf: &mut [u8] = &mut [0; 1024];
                    while let Ok(n) = f.read(buf) {
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
        resp.set_not_found().send();
    }
}
