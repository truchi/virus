use std::{io, str};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Message                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const CONTENT_LENGTH: &'static str = "Content-Length: ";
const CONTENT_TYPE: &'static str = "Content-Type: ";
const SEPARATOR: &'static str = "\r\n";

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Message {
    content_length: usize,
    content_type: Option<String>,
    content: Vec<u8>,
}

impl Message {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content_length: content.len(),
            content_type: None,
            content,
        }
    }

    pub fn with_content_type(content: Vec<u8>, content_type: String) -> Self {
        Self {
            content_length: content.len(),
            content_type: Some(content_type),
            content,
        }
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub async fn read<T: AsyncBufRead + Unpin>(reader: &mut T) -> io::Result<Self> {
        fn invalid_data(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
            io::Error::new(io::ErrorKind::InvalidData, err)
        }

        let lf = *SEPARATOR.as_bytes().last().expect("Separator last byte");
        let mut content = Vec::new();

        // Read content length
        reader.read_until(lf, &mut content).await?;

        if !content.starts_with(CONTENT_LENGTH.as_bytes())
            || !content.ends_with(SEPARATOR.as_bytes())
        {
            return Err(invalid_data("Expecting content length header"));
        }

        let content_length =
            str::from_utf8(&content[CONTENT_LENGTH.len()..content.len() - SEPARATOR.len()])
                .map_err(invalid_data)?
                .parse()
                .map_err(invalid_data)?;

        // Read optional content type
        content.clear();
        reader.read_until(lf, &mut content).await?;

        if !content.ends_with(SEPARATOR.as_bytes()) {
            return Err(invalid_data("Expecting optional content type header"));
        }

        let content_type = if content.len() == SEPARATOR.len() {
            None
        } else {
            if !content.starts_with(CONTENT_TYPE.as_bytes()) {
                return Err(invalid_data("Expecting content type header"));
            }

            let content_type =
                str::from_utf8(&content[CONTENT_TYPE.len()..content.len() - SEPARATOR.len()])
                    .map_err(invalid_data)?
                    .to_owned();

            // Read separator
            content.clear();
            reader.read_until(lf, &mut content).await?;

            if content != SEPARATOR.as_bytes() {
                return Err(invalid_data("Expecting header separator"));
            }

            Some(content_type)
        };

        // Read content
        content.clear();
        content.resize(content_length, 0);
        reader.read_exact(&mut content).await?;

        Ok(Self {
            content_length,
            content_type,
            content,
        })
    }

    pub async fn write<T: AsyncWrite + Unpin>(&self, writer: &mut T) -> io::Result<()> {
        debug_assert!(self.content.len() == self.content_length);

        let content_length = self.content_length.to_string();
        let content_type = self.content_type.as_ref();
        let bytes: Vec<_> = if let Some(content_type) = content_type {
            [
                CONTENT_LENGTH.as_bytes(),
                content_length.as_bytes(),
                SEPARATOR.as_bytes(),
                CONTENT_TYPE.as_bytes(),
                content_type.as_bytes(),
                SEPARATOR.as_bytes(),
                SEPARATOR.as_bytes(),
                &self.content,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect()
        } else {
            [
                CONTENT_LENGTH.as_bytes(),
                content_length.as_bytes(),
                SEPARATOR.as_bytes(),
                SEPARATOR.as_bytes(),
                &self.content,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect()
        };

        writer.write_all(&bytes).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn read_write() {
        let messages = [
            Message::new(Vec::new()),
            Message::new(Vec::from(b"Hello")),
            Message::with_content_type(Vec::new(), String::new()),
            Message::with_content_type(Vec::from(b"World"), String::from("some content type")),
        ];

        // One by one
        for message in &messages {
            let mut writer = Vec::new();
            message.write(&mut writer).await.expect("Write");

            let mut reader = BufReader::new(writer.as_slice());
            assert!(Message::read(&mut reader).await.expect("Read") == *message);
        }

        // All together
        let mut writer = Vec::new();
        for message in &messages {
            message.write(&mut writer).await.expect("Write");
        }

        let mut reader = BufReader::new(writer.as_slice());
        for message in &messages {
            assert!(Message::read(&mut reader).await.expect("Read") == *message);
        }
    }

    #[tokio::test]
    async fn errors() {
        let strings = [
            String::from(""),
            String::from("nonsense"),
            format!("{CONTENT_LENGTH}{SEPARATOR}"),
            format!("{CONTENT_LENGTH}{SEPARATOR}{SEPARATOR}"),
        ];

        for string in strings {
            let mut writer = Vec::new();
            writer.write(string.as_bytes()).await.unwrap();

            let mut reader = BufReader::new(writer.as_slice());
            assert!(Message::read(&mut reader).await.is_err());
        }
    }
}
