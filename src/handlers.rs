use http_file::*;
use http::*;
use handler_lib::*;

pub struct FileSystemHandler {
    path: String,
    fs: FileSystem,
}
impl FileSystemHandler {
    pub fn new(path: &str) -> FileSystemHandler {
        FileSystemHandler {
            path: path.to_string(),
            fs: FileSystem::new(path),
        }
    }
}
impl Handler for FileSystemHandler {
    fn process(&mut self, req: Request, resp: &mut Response) {
        self.fs.serve(&req.uri, resp);
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(FileSystemHandler::new(&self.path));
    }
}

pub struct FileHandler {
    path: String,
    fs: FileSystem,
}
impl FileHandler {
    pub fn new(path: &str) -> FileHandler {
        FileHandler {
            path: path.to_string(),
            fs: FileSystem::new(path),
        }
    }
}
impl Handler for FileHandler {
    fn process(&mut self, _: Request, resp: &mut Response) {
        self.fs.serve("", resp);
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(FileHandler::new(&self.path));
    }
}
