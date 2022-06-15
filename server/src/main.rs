mod buffer;
mod ray;
mod routes;
mod texture;
mod world;

use texture::Textures;

use clap::Parser;
use std::sync::Arc;
use warp::Filter;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// The host we're listening on.
  #[clap(long, default_value = "127.0.0.1")]
  host: std::net::IpAddr,

  /// The port this server is hosted on.
  #[clap(long, default_value_t = 8080)]
  port: u16,
}

fn with_context<T: Sync + Send>(
  obj: T,
) -> impl Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone {
  let obj = Arc::new(obj);
  warp::any().map(move || obj.clone())
}

#[tokio::main]
async fn main() {
  {
    // Logging initialisation. Roll some custom stuff to default to info - not sure how better to handle this!
    let mut builder = pretty_env_logger::formatted_builder();
    if let Ok(filter) = std::env::var("RUST_LOG") {
      builder.parse_filters(&filter);
    } else {
      builder.parse_filters("info");
    }
    builder.init();
  }

  let args = Args::parse();

  let textures = with_context(Textures::new().unwrap());

  let metrics = warp::path("metrics").map(routes::metrics);

  let render = warp::path("render")
    .and(warp::ws())
    .and(textures.clone())
    .map(routes::render);

  warp::serve(metrics.or(render))
    .run((args.host, args.port))
    .await;
}
