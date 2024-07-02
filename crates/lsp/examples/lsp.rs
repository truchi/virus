use lsp::protocol::lsp::Message;
use lsp::protocol::rpc::Id;
use lsp::{client::Client, protocol::rpc::Response};
use serde::{Deserialize, Serialize};
use tokio::io::{duplex, AsyncWriteExt, BufReader, DuplexStream};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    #[derive(Serialize, Deserialize, Debug)]
    struct TestType {
        field: String,
    }

    let (_reader1, writer1) = duplex(1024);
    let (reader2, mut writer2) = duplex(1024);
    let reader2 = BufReader::new(reader2);

    let (sender, receiver) = channel(1024);

    let mut client = Client::new(reader2, writer1, sender).unwrap();

    let response = client
        .request::<_, TestType>(String::from("req"), Some("lalal"))
        .await
        .unwrap();
    println!("ici");

    Message::new(
        serde_json::to_vec(&Response {
            jsonrpc: "2.0".into(),
            id: Some(Id::Integer(0)),
            result: Some(TestType {
                field: String::from("result"),
            }),
            error: None,
        })
        .unwrap(),
    )
    .write(&mut writer2)
    .await
    .unwrap();

    let response = response.await;
    println!("la");
    dbg!(response);
}

#[tokio::main]
async fn main2() {
    let (reader, mut writer) = duplex(1024);

    let handle = tokio::spawn(async move {
        let mut reader = BufReader::new(reader);

        for i in 0..10 {
            let message = Message::<Vec<u8>>::read(&mut reader).await.unwrap();
            let message = Message {
                headers: message.headers,
                content: String::from_utf8(message.content).unwrap(),
            };
            println!("---------- GOT MESSAGE {i}: {message:#?}");
        }
    });

    async fn write(writer: &mut DuplexStream, str: &str) {
        println!("Writing: {str:#?}");
        writer.write_all(str.as_bytes()).await.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    write(&mut writer, "Content-").await;
    write(&mut writer, "Length: 10\r\n").await;
    write(&mut writer, "Content-").await;
    write(&mut writer, "Type: blabla\r\n").await;
    write(&mut writer, "\r\n").await;
    write(&mut writer, "0123456789").await;

    write(&mut writer, "Content-Length: 10\r\n").await;
    write(&mut writer, "\r\n!!!!!!!!!!").await;

    Message::new(String::from("Lol").into_bytes())
        .write(&mut writer)
        .await
        .unwrap();

    write(
        &mut writer,
        "Content-Length: 10\r\n\r\naaaaaaaaaaContent-Length: 10\r\n\r\nbbbbbbbbbb",
    )
    .await;

    Message::new(String::from("").into_bytes())
        .write(&mut writer)
        .await
        .unwrap();

    Message::new(String::from("").into_bytes())
        .write(&mut writer)
        .await
        .unwrap();

    // drop(writer);

    handle.await.unwrap();
}
