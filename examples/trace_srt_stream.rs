use srt::server::Server;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    Server::new().run("0.0.0.0:9000")
}
