use futures::Stream;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::rpc::Response;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         ContentLength                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Debug)]
pub struct ContentLength(pub usize);

impl ContentLength {
    pub const PREFIX: &'static str = "Content-Length: ";
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          ContentType                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct ContentType(pub String);

impl ContentType {
    pub const PREFIX: &'static str = "Content-Type: ";
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Headers                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Headers {
    pub content_length: ContentLength,
    pub content_type: Option<ContentType>,
}

impl Headers {
    pub const SEPARATOR: &'static str = "\r\n";
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Message                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, Debug)]
pub struct Message<T> {
    pub headers: Headers,
    pub content: T,
}

impl<T> Message<T> {
    pub fn new(content: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            headers: Headers {
                content_length: ContentLength(content.as_ref().len()),
                content_type: None,
            },
            content,
        }
    }

    pub fn new_with_type(content: T, content_type: String) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            headers: Headers {
                content_length: ContentLength(content.as_ref().len()),
                content_type: Some(ContentType(content_type)),
            },
            content,
        }
    }
}

impl Message<Vec<u8>> {
    pub async fn read<T: AsyncBufRead + Unpin>(reader: &mut T) -> std::io::Result<Self> {
        let mut content = Vec::new();
        let lf = *Headers::SEPARATOR
            .as_bytes()
            .last()
            .expect("Separator last byte");

        // Read content length
        reader.read_until(lf, &mut content).await?;
        debug_assert!(content.starts_with(ContentLength::PREFIX.as_bytes()));
        debug_assert!(content.ends_with(Headers::SEPARATOR.as_bytes()));

        let content_length = ContentLength(
            std::str::from_utf8(
                &content[ContentLength::PREFIX.len()..content.len() - Headers::SEPARATOR.len()],
            )
            .expect("UTF-8")
            .parse()
            .expect("Number"),
        );

        content.clear();

        // Read optional content type
        reader.read_until(lf, &mut content).await?;
        debug_assert!(content.ends_with(Headers::SEPARATOR.as_bytes()));

        let content_type = if content.len() == Headers::SEPARATOR.len() {
            None
        } else {
            debug_assert!(content.starts_with(ContentType::PREFIX.as_bytes()));

            let content_type = ContentType(
                std::str::from_utf8(
                    &content[ContentType::PREFIX.len()..content.len() - Headers::SEPARATOR.len()],
                )
                .expect("UTF-8")
                .to_owned(),
            );

            content.clear();

            // Read separator
            reader.read_until(lf, &mut content).await?;
            debug_assert!(content == Headers::SEPARATOR.as_bytes());

            Some(content_type)
        };

        content.clear();

        // Read content
        content.resize(content_length.0, 0);
        reader.read_exact(&mut content).await?;

        Ok(Self {
            headers: Headers {
                content_length,
                content_type,
            },
            content,
        })
    }

    pub async fn write<T: AsyncWrite + Unpin>(&self, writer: &mut T) -> std::io::Result<()> {
        debug_assert!(self.content.len() == self.headers.content_length.0);

        let content_length = self.headers.content_length.0.to_string();
        let content_type = self
            .headers
            .content_type
            .as_ref()
            .map(|content_type| &content_type.0);
        let bytes = if let Some(content_type) = content_type {
            [
                ContentLength::PREFIX.as_bytes(),
                content_length.as_bytes(),
                Headers::SEPARATOR.as_bytes(),
                ContentType::PREFIX.as_bytes(),
                content_type.as_bytes(),
                Headers::SEPARATOR.as_bytes(),
                Headers::SEPARATOR.as_bytes(),
                &self.content,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
        } else {
            [
                ContentLength::PREFIX.as_bytes(),
                content_length.as_bytes(),
                Headers::SEPARATOR.as_bytes(),
                Headers::SEPARATOR.as_bytes(),
                &self.content,
            ]
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
        };

        writer.write_all(&bytes).await
    }
}
