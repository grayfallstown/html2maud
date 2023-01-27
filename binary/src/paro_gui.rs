#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::str::FromStr;

use tauri;
// use tauri_plugin_websocket::TauriWebsocket;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async};
use tungstenite::{Result, Message};

use clipboard::{ClipboardProvider, ClipboardContext};
use maud::{html};

use paro_rs::{ParoApp, event};

use html2maud::html2maud;

use rust_embed::*;
use tiny_http::*;

#[derive(RustEmbed)]
#[folder = "./public/"]
struct Asset;



pub struct ApplicationState {
    pub html: String,
    pub maud: String,
}


fn render(paro_app: &mut Arc<Mutex<ParoApp<ApplicationState>>>) -> String {
    let markup = html! {
        div #mainContainer.container {
            div.row {
                div.col {
                    div."mb-3" {
                        label."form-label" for="htmlTextarea" {
                        "HTML input"
                        }
                        textarea #htmlTextarea."form-control" autofocus oninput=(
                            event!(paro_app, (move |state: &mut ApplicationState, value| {
                                println!("value: {}", value);
                                state.html = value;
                                state.maud = html2maud(&state.html);    
                            }))
                        ) {
                            (paro_app.lock().unwrap().state.html)
                        }
                    }
                }
     
                div.col {
                   button.btn."btn-primary" type="button" onclick=(
                        event!(paro_app, (move |state: &mut ApplicationState, _| {
                            let mut clipboard_context: ClipboardContext = ClipboardProvider::new().unwrap();
                            clipboard_context.set_contents(state.maud.clone())
                                .expect("could not write to clipboard");
                        }))
                    ) {
                        "Copy to Clipboard"
                    }
                    pre {
                        (paro_app.lock().unwrap().state.maud)
                    }
                }
            }
        }
    };

    let html = markup.into_string();
    return html;
}


/**
 * Start a websocket server for pâro to connect to
 */
async fn start_server() {
    let addr = "127.0.0.1:7437".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");

    let html = r#"<h1>Welcome to html2maud</h1>
    <p>Paste or type <strong>html</strong> into the left side</p>
    <br/>
    <p>and the <strong>maud template</strong> will be shown on the right side</p>"#.to_owned();
    let paro_app = Arc::new(Mutex::new(ParoApp::<ApplicationState>::new(ApplicationState {
        maud: html2maud::html2maud(&html),
        html: html,
    })));

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        tokio::spawn(accept_connection(paro_app.clone(), peer, stream));
    }
}

async fn accept_connection(paro_app: Arc<Mutex<ParoApp<ApplicationState>>>, peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(paro_app, peer, stream).await {
        match e {
            err => println!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(paro_app: Arc<Mutex<ParoApp<ApplicationState>>>, _peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

     // initial html
    let rendered_html = render(&mut paro_app.clone());
    ws_stream.send(Message::Text(rendered_html)).await?;
    
    // You can have an eventloop here to match pâro message input, database returns result,
    // async api calls, etc

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            if msg.is_text() && msg.to_text().unwrap().eq("ping") {
                // ping / pong to keep the websocket alive while the user if afk
                ws_stream.send(Message::Text("pong".to_owned())).await?;
            } else {
                let event_id = msg.to_text().unwrap();
                println!("calling pâro event id '{}'", &event_id);
                paro_app.lock().unwrap().call(event_id.to_owned())
                    .expect(&format!("could not call paro callback for id '{}'", event_id));

                // clean up old callbacks to free memory
                paro_app.lock().unwrap().iterate();
                // render updated html and fill callbackstore with current callbacks
                let rendered_html = render(&mut paro_app.clone());
                // send updated html to the client, so it can be shown to the user
                ws_stream.send(Message::Text(rendered_html)).await?;
            }
        }
    }

    Ok(())
}


fn serve_assets(port: usize, get_file: Box<dyn Fn(&str) -> Option<EmbeddedFile> + 'static>) {
    let server = Server::http(&format!("127.0.0.1:{}", port)).unwrap();
    let mime_types = mime_types();

    for request in server.incoming_requests() {
        let url = request.url();
        let path = url.trim_start_matches('/');
        let asset = get_file(path).or(get_file("index.html"));
        match asset {
            Some(embedded_file) => {
                let ext = if path.is_empty() { "html" } else { path.split(".").last().unwrap() };
                println!("path: {}, Extension: {}", path, ext);
                let mime = mime_types.get(ext).unwrap_or(&"application/octet-stream");
                let response = Response::from_data(embedded_file.data)
                    .with_header(Header::from_str(&format!("Content-Type:{}", *mime)).unwrap());
                _ = request.respond(response);
            },
            None => { 
                _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            },
        }
    }
}

fn mime_types() -> HashMap<&'static str, &'static str> {
    let mut mime_types = HashMap::new();
    mime_types.insert("html", "text/html");
    mime_types.insert("htm", "text/html");
    mime_types.insert("css", "text/css");
    mime_types.insert("js", "application/javascript");
    mime_types.insert("png", "image/png");
    mime_types.insert("jpg", "image/jpeg");
    mime_types.insert("jpeg", "image/jpeg");
    mime_types.insert("webp", "image/webp");
    mime_types.insert("gif", "image/gif");
    mime_types.insert("svg", "image/svg+xml");
    mime_types
}


pub(crate) fn start_gui() {
    tauri::async_runtime::spawn(start_server());
    tauri::Builder::default()
        .setup(|app| {
            //tauri::async_runtime::spawn(|| {
            std::thread::spawn(move || {
                serve_assets(8080, Box::new(|path| { Asset::get(path) }));
            });
            Ok(())
        })
        // .plugin(TauriWebsocket::default()) // this was added
        .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
