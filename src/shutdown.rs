/**
* filename : shutdown
* author : HAMA
* date: 2025. 4. 6.
* description: 
**/

use tokio::signal;

pub async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("Failed to install Ctrl+C handler");
  };
  
  #[cfg(unix)]
  let terminate = async {
    use tokio::signal::unix::{signal, SignalKind};
    let mut stream = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
    stream.recv().await;
  };
  
  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();
  
  tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
  
  println!("Shutdown signal received.");
}
