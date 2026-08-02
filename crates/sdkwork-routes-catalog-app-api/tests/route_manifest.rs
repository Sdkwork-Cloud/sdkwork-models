use sdkwork_routes_models_catalog_app_api::{app_route_manifest, route_definitions};
use sdkwork_web_core::RouteAuth;

#[test]
fn app_route_manifest_matches_route_definitions() {
    let manifest = app_route_manifest();
    for entry in route_definitions() {
        let matched = manifest
            .match_route(entry.method, entry.path)
            .unwrap_or_else(|| {
                panic!(
                    "missing http route manifest for {} {}",
                    entry.method, entry.path
                )
            });
        // The app catalog surface is only consumed by authenticated clients
        // (the SDK always attaches dual-token credentials), so every route is
        // dual-token; anonymous classification would reject those credentials.
        assert_eq!(matched.auth, RouteAuth::DualToken);
        assert_eq!(matched.operation_id, entry.operation_id);
    }
}
