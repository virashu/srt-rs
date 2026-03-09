use srt::CallbackListener;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    CallbackListener::new().run("0.0.0.0:1935")
}
