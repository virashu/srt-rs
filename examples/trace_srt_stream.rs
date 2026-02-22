use srt::CallbackServer;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    CallbackServer::new().run("0.0.0.0:9000")
}
