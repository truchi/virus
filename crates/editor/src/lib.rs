use ropey::Rope;

#[derive(Clone, Default, Debug)]
pub struct Document {
    path: Option<String>,
    rope: Rope,
}

impl Document {
    pub fn open(path: String) -> std::io::Result<Self> {
        use std::{fs::File, io::BufReader};

        let rope = Rope::from_reader(&mut BufReader::new(File::open(&path)?))?;

        Ok(Self {
            path: Some(path),
            rope,
        })
    }
}
