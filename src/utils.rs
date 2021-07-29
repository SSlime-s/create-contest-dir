use std::{
    fs::File,
    io::{Error, Read, Seek, SeekFrom},
};

pub fn clear_file(file: &mut File) -> Result<String, Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(content)
}
