#[tokio::main]
async fn main() -> std::process::ExitCode {
    match mcp_gmap_rs::run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
