#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

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
                        textarea #htmlTextarea."form-control" autofocus oninput=({
                            event!(paro_app, (move |state: &mut ApplicationState, value| {
                                println!("value: {}", value);
                                state.html = value;
                                state.maud = html2maud(&state.html);    
                            }))
                        }) {
                            (paro_app.lock().unwrap().state.html)
                        }
                    }
                }
     
                div.col {
                   button.btn."btn-primary" type="button" onclick=({
                        event!(paro_app, (move |state: &mut ApplicationState, _| {
                            let mut clipboard_context: ClipboardContext = ClipboardProvider::new().unwrap();
                            clipboard_context.set_contents(state.maud.clone())
                                .expect("could not write to clipboard");
                        }))
                    }) {
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
    println!("maud generated html:\n{}", html);
    return html;
}


/**
 * Start a websocket server for pâro to connect to
 */
async fn start_server() {
    let addr = "127.0.0.1:7437".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");

    let paro_app = Arc::new(Mutex::new(ParoApp::<ApplicationState>::new(ApplicationState {
        html: "<p>Paste or type html into the left side</p>".to_owned(),
        maud: html2maud::html2maud("<p>Paste or type html into the left side</p>"),
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


pub(crate) fn start_gui() {
    tauri::async_runtime::spawn(start_server());
    tauri::Builder::default()
        // .plugin(TauriWebsocket::default()) // this was added
        .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
