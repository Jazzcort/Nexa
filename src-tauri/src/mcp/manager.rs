use std::collections::HashMap;

use crate::mcp::connection;
use serde_json::json;
use std::pin::pin;

// async fn testing() {
//     let mut manager: HashMap<String, Box<dyn connection::MCPConnection>> = HashMap::new();
//
//     manager.insert(
//         "function1".to_string(),
//         Box::new(connection::MCPStdioConnection::new()),
//     );
//
//     if let Some(conn) = manager.get("function1") {
//         let a = conn.call_tool(None);
//         let result = a.await;
//         dbg!(result);
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn test_manager() {
    //     let _ = testing().await;
    // }
}
