//! A local compatibility host using the platform checkout's runtime crate.
//! It runs no NATS/control-plane services and is not a production deployment.
use common::{
    policy::{NetworkPolicyConfig, PolicyConfig},
    types::{AppConfig, AppId, FuelQuota, MemoryPages},
};
use runtime::WasmRuntime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("runtime=warn")
        .with_ansi(false)
        .init();
    let args: Vec<_> = std::env::args().collect();
    let artifact = std::fs::read(args.get(1).ok_or("component path required")?)?;
    let port: u16 = args.get(2).ok_or("port required")?.parse()?;
    let runtime = WasmRuntime::new()?;
    let artifact = runtime.compile(&artifact)?;
    let mut config = AppConfig::default_for(AppId::new("djmx-media-worker", "validation"));
    config.fuel_quota = FuelQuota(
        std::env::var("WCP_PROBE_FUEL")
            .unwrap_or_else(|_| "10000000000".into())
            .parse()?,
    );
    config.memory_limit = MemoryPages(8192);
    config.max_instances = 1;
    config.wasm_bind_port = port;
    config.policy = Some(PolicyConfig {
        network: Some(NetworkPolicyConfig {
            allow_inbound: Some(true),
            allow_outbound_tcp: Some(true),
            allow_dns: Some(true),
            allowed_cidrs: Some(vec!["127.0.0.0/8".into()]),
            ..Default::default()
        }),
        ..Default::default()
    });
    let env = std::env::vars()
        .filter(|(key, _)| {
            key == "DATABASE_URL" || key.starts_with("STORAGE_") || key.starts_with("MEDIA_")
        })
        .collect();
    let prepared = runtime.prepare(&artifact, config)?;
    let mut server = prepared.spawn_http_server(env, ([127, 0, 0, 1], port).into(), None, None)?;
    eprintln!("Platform WASI HTTP runtime listening on port {port}; no filesystem preopens");
    tokio::select! {
        result=&mut server.task=>{let stats=result?;if let Some(trap)=stats.trap {return Err(trap.into());}},
        _=tokio::signal::ctrl_c()=>{let _=server.shutdown_tx.send(());let _=server.task.await;}
    }
    Ok(())
}
