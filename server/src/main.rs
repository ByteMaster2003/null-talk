use std::sync::Arc;

use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};

type ClientTx = mpsc::Sender<String>;
struct Client {
    tx: ClientTx,
}
type PeerMap = Arc<DashMap<String, Client>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr.clone()).await?;

    let peers: PeerMap = Arc::new(DashMap::new());

    println!("server is running on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let peers_clone = peers.clone();

        tokio::spawn(async move { handle_client(stream, peers_clone).await });
    }
}

async fn handle_client(stream: TcpStream, peers: PeerMap) {
    // step 1: create lines frame fram tokio_util
    let mut lines = Framed::new(stream, LinesCodec::new());

    // step 2: handshanke,
    // ask for name
    lines.send("enter your name:").await.unwrap();
    let user_id = match lines.next().await {
        Some(Ok(line)) => line,
        _ => return,
    };

    // step 3: create channel (Mail Box or Queue) for the client
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // step 4: register the user in Dash map
    peers.insert(user_id.clone(), Client { tx });

    let (mut sink, mut stream) = lines.split();

    // Task A writer task
    // read the message from channel and forward it to the client
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(_) = sink.send(msg).await {
                break; // client is probalby disconnected
            }
        }
    });

    // Task B Reader task
    // wait for the clients message and forward the message to the receivers channel
    while let Some(result) = stream.next().await {
        match result {
            Ok(line) => {
                // Let's assume
                // protocol as user_id:msg
                if let Some((dest_id, msg)) = line.split_once(":") {
                    if let Some(receiver) = peers.get(dest_id) {
                        let _ = receiver.tx.send(format!("[{}]: {}", user_id, msg)).await;
                    } else {
                        let _ = peers
                            .get(&user_id)
                            .unwrap()
                            .tx
                            .send(format!("[{}] is not connected", dest_id))
                            .await;
                    }
                }
            }
            _ => break, // It means client disconneted
        };
    }

    writer_task.abort();
    peers.remove(&user_id);
    println!("{} is disconnected", user_id)
}
