use anyhow::Result;
use clap::Parser;
use common::output::Output;
use common::status::Status;
use futures::StreamExt;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use zbus::{Connection, proxy};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: Option<String>,
}

#[proxy(
    default_service = "org.mtmn.Plants",
    default_path = "/org/mtmn/Plants",
    interface = "org.mtmn.Plants"
)]
trait PlantsDaemon {
    #[zbus(signal)]
    async fn update(&self, status: Status);
}

type SharedOutput = Arc<RwLock<String>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let output_state = Arc::new(RwLock::new(String::new()));

    let initial = Output::not_connected();
    initial.print();

    // Spawn DBus listener task
    let output_clone = output_state.clone();
    let mut dbus_handle = tokio::spawn(async move {
        if let Err(e) = listen_dbus(output_clone).await {
            eprintln!("DBus listener error: {e}");
        }
    });

    // Create web server if host is provided
    if let Some(host) = args.host {
        let listener = TcpListener::bind(&host).await?;

        loop {
            // Wait for either the dbus listener (which shouldn't exit) or a new connection
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((socket, _)) => {
                            let state = output_state.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(socket, state).await {
                                    eprintln!("Error handling connection: {e}");
                                }
                            });
                        }
                        Err(e) => eprintln!("Accept error: {e}"),
                    }
                }
                _ =  &mut dbus_handle => {
                    // DBus handle finished
                    break;
                }
            }
        }
    } else {
        // If no host is defined, just await the dbus task
        if let Err(e) = dbus_handle.await {
            eprintln!("DBus task error: {e}");
        }
    }

    Ok(())
}

async fn handle_connection(socket: tokio::net::TcpStream, state: SharedOutput) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let json = state.read().await.clone();
    let json = if json.is_empty() {
        "{}".to_string()
    } else {
        json
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         \r\n\
         {}",
        json.len(),
        json
    );

    writer.write_all(response.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

async fn listen_dbus(output_state: SharedOutput) -> Result<()> {
    let connection = Connection::session().await?;
    let proxy = PlantsDaemonProxy::new(&connection).await?;
    let mut stream = proxy.receive_update().await?;

    while let Some(msg) = stream.next().await {
        let status = msg.args()?.status;

        let merged_output = Output::from_status(&status);
        merged_output.print();

        let json = serde_json::to_string(&merged_output).unwrap_or_else(|_| "{}".to_string());
        let mut state = output_state.write().await;
        *state = json;
    }

    Ok(())
}
