#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteDefinition {
    pub method: &'static str,
    pub path: &'static str,
    pub operation_id: &'static str,
    pub handler: &'static str,
    pub service_method: &'static str,
}

pub const ROUTES: &[RouteDefinition] = &[
    RouteDefinition {
        method: "GET",
        path: "/app/v3/api/ai/model_vendors",
        operation_id: "modelVendors.list",
        handler: "list_model_vendors",
        service_method: "list_model_vendors",
    },
    RouteDefinition {
        method: "GET",
        path: "/app/v3/api/ai/models",
        operation_id: "models.list",
        handler: "list_models",
        service_method: "list_models",
    },
    RouteDefinition {
        method: "GET",
        path: "/app/v3/api/ai/model_rankings",
        operation_id: "modelRankings.list",
        handler: "list_model_rankings",
        service_method: "list_model_rankings",
    },
];

pub fn route_definitions() -> &'static [RouteDefinition] {
    ROUTES
}
