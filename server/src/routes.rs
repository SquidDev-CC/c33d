//! The various routes served by c33d.

use prometheus::{Encoder, TextEncoder};
use warp::http::{header::CONTENT_TYPE, Response};
use warp::hyper::Body;

/// `GET /metrics`: Exports Prometheus metrics.
pub fn metrics() -> Response<Body> {
  let encoder = TextEncoder::new();
  let metric_families = prometheus::gather();
  let mut buffer = vec![];
  encoder.encode(&metric_families, &mut buffer).unwrap();

  Response::builder()
    .status(200)
    .header(CONTENT_TYPE, encoder.format_type())
    .body(Body::from(buffer))
    .unwrap()
}

mod render {
  use futures_util::{SinkExt, StreamExt};
  use lazy_static::lazy_static;
  use log::error;
  use prometheus::{register_histogram, Histogram};
  use serde::de::DeserializeOwned;
  use serde::Deserialize;
  use std::sync::Arc;
  use warp::ws::Message;
  use warp::Reply;

  use crate::ray::{render as do_render, Vec3};
  use crate::texture::Textures;
  use crate::world::World;

  lazy_static! {
    static ref RENDER_DURATION: Histogram = register_histogram!(
      "r3d_render_duration_seconds",
      "The time taken to render a single scene",
      vec![0.0, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1],
    )
    .unwrap();
  }

  #[derive(Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct WorldMessage {
    world: World,
    offset_x: f64,
    offset_y: f64,
    offset_z: f64,
  }

  #[derive(Deserialize)]
  struct RenderMessage {
    x: f64,
    y: f64,
    z: f64,
  }

  fn decode_message<T: DeserializeOwned>(msg: Result<Message, warp::Error>) -> Option<T> {
    match msg {
      Ok(msg) => match msg.to_str() {
        Ok(msg) => match serde_json::from_str(msg) {
          Ok(result) => Some(result),
          Err(err) => {
            error!("Failed to parse message: {}", err);
            None
          }
        },
        Err(()) => {
          error!("Failed to parse message: Not a binary message.");
          None
        }
      },
      Err(e) => {
        error!("Error in receiving message: {}", e);
        None
      }
    }
  }

  async fn websocket_handler(websocket: warp::ws::WebSocket, textures: Arc<Textures>) {
    let (mut send, mut receive) = websocket.split();

    let world = if let Some(world) = receive
      .next()
      .await
      .and_then(decode_message::<WorldMessage>)
    {
      world
    } else {
      return;
    };

    while let Some(message) = receive.next().await {
      if let Some(position) = decode_message::<RenderMessage>(message) {
        let timer = RENDER_DURATION.start_timer();

        let buffer = do_render(
          &world.world,
          &textures,
          Vec3::new(world.offset_x, world.offset_y, world.offset_z),
          Vec3::new(position.x, position.y, position.z),
        );
        let result = buffer.draw();

        timer.observe_duration();

        if let Err(err) = send.send(Message::binary(result)).await {
          error!("Error sending message: {}", err);
        }
      }
    }
  }

  /// `GET /render`: Serves a websocket which accepts messages of the form `{ x: f64, y: f64, z: f64 }` and responds
  /// with the rendered world.
  pub fn render(ws: warp::ws::Ws, textures: Arc<Textures>) -> impl Reply {
    ws.on_upgrade(move |websocket| websocket_handler(websocket, textures))
  }
}

pub use render::render;
