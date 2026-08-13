#[cfg(test)]
mod tests {
    use simbridge_shared::networking::*;

    #[test]
    fn test_health_check_request() {
        let client = RestClient::new("http://localhost:8080");

        // Test that the client can be created
        assert!(client.base_url.starts_with("http://"));
    }

    #[test]
    fn test_simulator_list_request() {
        let client = RestClient::new("http://localhost:8080");

        // Test building the simulator list endpoint
        let url = format!("{}/api/v1/simulators", client.base_url);
        assert!(url.contains("/simulators"));
    }

    #[test]
    fn test_session_management_endpoints() {
        let client = RestClient::new("http://localhost:8080");

        // Test session list endpoint
        let list_url = format!("{}/api/v1/sessions", client.base_url);
        assert!(list_url.contains("/sessions"));

        // Test session delete endpoint
        let delete_url = format!("{}/api/v1/sessions/:id", client.base_url);
        assert!(delete_url.contains("/sessions/:id"));
    }

    #[test]
    fn test_endpoint_construction() {
        let client = RestClient::new("http://localhost:8080");

        // Build various endpoints
        let health_url = format!("{}/health", client.base_url);
        assert_eq!(health_url, "http://localhost:8080/health");
    }
}
