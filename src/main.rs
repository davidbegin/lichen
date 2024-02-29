use nannou::prelude::*;
use palette::named;
use tokio::sync::mpsc;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

fn main() {
     nannou::app(model)
        .event(event)
        .view(view)
        .run();
}

enum PaintBrush {
    Ball,
    Line,
    FunLine,
    FunBall,
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    receiver: mpsc::Receiver<ServerMessage>,
    color: rgb::Srgb<u8>,
    paintbrush_size: f32,
    paint_brush: PaintBrush,
    start: Point2,
    end: Point2,
    // line_b: Point
    // line_a:
}

fn model(app: &App) -> Model {
    let (sender, receiver) = mpsc::channel::<ServerMessage>(32);

    let _window = app
        .new_window()
        .size(512, 512)
        .title("nannou")
        .view(view) // The function that will be called for presenting graphics to a frame.
        .build()
        .unwrap();

    tokio::spawn(async move {
        kick_off_twitch_chat(sender).await;
    });

    Model {
        _window,
        receiver,
        color: CORNFLOWERBLUE,
        paintbrush_size: 0.3,
        paint_brush: PaintBrush::Line,
        start: app.window_rect().top_left(),
        end: app.window_rect().bottom_right(),
    }
}

// This is for handling mouse movements and other window interactions
// Handle events related to the window and update the model if necessary
// fn event(_app: &App, _model: &mut Model, _event: WindowEvent) {
fn event(_app: &App, _model: &mut Model, _event: Event) {
    if let Ok(message) = _model.receiver.try_recv() {
        let params = &message.source().params;
        if params.len() > 1 {
            let command = &params[1];
            match command.as_str() {
                "v" => {
                    _model.start = _app.window_rect().mid_top() * _app.mouse.x;
                    _model.end = _app.window_rect().mid_bottom() * _app.mouse.y;
                }
                "h" => {
                    _model.start = _app.window_rect().mid_left() * _app.mouse.x;
                    _model.end = _app.window_rect().mid_right() * _app.mouse.y;
                }
                "fs" => {
                    _model.start = _app.window_rect().top_right() * _app.mouse.x;
                    _model.end = _app.window_rect().bottom_left() * _app.mouse.y;
                }
                "bs" => {
                    _model.start = _app.window_rect().top_left() * _app.mouse.x;
                    _model.end = _app.window_rect().bottom_right() * _app.mouse.y;
                }
                "big" => {
                    _model.paintbrush_size = _model.paintbrush_size * 2.0;
                }
                "small" => {
                    _model.paintbrush_size = _model.paintbrush_size * 0.5;
                }
                "line" => {
                    _model.paint_brush = PaintBrush::Line;
                }
                "ball" => {
                    _model.paint_brush = PaintBrush::Ball;
                }
                "funline" => {
                    _model.paint_brush = PaintBrush::FunLine;
                }
                "funball" => {
                    _model.paint_brush = PaintBrush::FunBall;
                }
                "plum" => {
                    _model.color = PLUM;
                }
                _ => {
                    if let Some(color) = named::from_str(command) {
                        let new_color = rgb::Srgb::new(color.red, color.green, color.blue);
                        _model.color = new_color;
                    } else {
                        // // if one of the positions isn't parseable
                        // // we will return an error and the update will fail
                        // // potentially with out the user nowing
                        // let rgb_vals: Vec<u8> = command
                        //     .split_whitespace()
                        //     .filter_map(|x| x.parse::<u8>().ok())
                        //     .collect();
                        //
                        // // If we filter out a value, we will have less than 3
                        // // however you could supply a chat message: 2 255 dog 255
                        // if rgb_vals.len() != 3 {
                        //     return;
                        // }
                        //
                        // let red = rgb_vals[0];
                        // let green = rgb_vals[1];
                        // let blue = rgb_vals[2];
                        // let new_color = rgb::Srgb::new(red, green, blue);
                        // _model.color = new_color;

                        // I don't need to do this
                        // println!("\nTwitch Message: {:?}", message.source().params[1]);
                    }
                }
            }
        }
    }

    // Respond to mouse event
    // println!("\n{:?}", event);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, _model: &Model, frame: Frame) {
    // I would read in details
    let draw = _app.draw();
    // draw.background().color(_model.color);

    // Want this to turn off when I call another command
    let win = _app.window_rect();
    let t = _app.time;

    match _model.paint_brush {
        PaintBrush::Ball => {
            draw.ellipse()
                .x_y(_app.mouse.x, _app.mouse.y)
                .radius(_model.paintbrush_size)
                // .radius(win.w() * _model.paintbrush_size)
                .color(_model.color);
        }
        PaintBrush::FunBall => {
            draw.ellipse()
                .x_y(_app.mouse.x, _app.mouse.y)
                .radius(win.w() * _model.paintbrush_size * t.sin())
                .color(_model.color);
        }
        PaintBrush::Line => {
            draw.line()
                .weight(_model.paintbrush_size)
                .caps_round()
                .color(_model.color)
                .x_y(_app.mouse.x, _app.mouse.y)
                .points(_model.start, _model.end);
        }
        PaintBrush::FunLine => {
            draw.line()
                .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
                .caps_round()
                .color(_model.color)
                .x_y(_app.mouse.x, _app.mouse.y)
                .points(win.top_left() * _app.mouse.x, win.bottom_right() * t.cos());
        }
    }

    // I could supress this output
    // thats bad
    // but it might help
    // This to frame spits out soo many INFO messages
    
    // Why is this spamming us!!!!
    draw.to_frame(_app, &frame).unwrap();
}

// receiver: mpsc::Receiver<String>,
async fn kick_off_twitch_chat(sender: mpsc::Sender<ServerMessage>) {
    tracing_subscriber::fmt::init();

    // default configuration is to join chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            tracing::info!("\nReceived message: {:?}", message);
            let _ = sender.send(message).await;
        }
    });

    // join a channel
    // This function only returns an error if the passed channel login name is malformed,
    // so in this simple case where the channel name is hardcoded we can ignore the potential
    // error with `unwrap`.
    client.join("beginbot".to_owned()).unwrap();

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}
