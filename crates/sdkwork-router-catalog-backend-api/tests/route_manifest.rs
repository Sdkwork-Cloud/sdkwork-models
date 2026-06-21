use sdkwork_router_catalog_backend_api::{backend_route_manifest, route_definitions};
use sdkwork_web_core::RouteAuth;

#[test]
fn backend_route_manifest_matches_route_definitions() {
    let manifest = backend_route_manifest();
    for entry in route_definitions() {
        let matched = manifest
            .match_route(entry.method, entry.path)
            .unwrap_or_else(|| {
                panic!(
                    "missing http route manifest for {} {}",
                    entry.method, entry.path
                )
            });
        assert_eq!(matched.auth, RouteAuth::DualToken);
        assert_eq!(matched.operation_id, entry.operation_id);
    }
}
