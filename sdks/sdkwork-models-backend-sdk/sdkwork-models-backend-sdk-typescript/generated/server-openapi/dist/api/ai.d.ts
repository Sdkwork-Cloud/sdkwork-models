import type { HttpClient } from '../http/client';
import type { AdminAccountModelMappingsReplaceRequest, AdminAiModelCreateRequest, AdminAiModelUpdateRequest, AdminAiResourceCreateRequest, AdminAiResourceGroupCreateRequest, AdminAiResourceGroupUpdateRequest, AdminAiResourceUpdateRequest, AdminModelCatalogSyncRequest, AdminModelMappingCreateRequest, AdminModelMappingResolveRequest, AdminModelMappingUpdateRequest, AdminModelVendorCreateRequest, AiResourceGroupsCreateResult, AiResourceGroupsDeleteResult, AiResourceGroupsListResult, AiResourceGroupsResourcesListResult, AiResourceGroupsUpdateResult, AiResourcesCreateResult, AiResourcesListResult, AiResourcesUpdateResult, ModelMappingsCreateResult, ModelMappingsDeleteResult, ModelMappingsListResult, ModelMappingsReplaceResult, ModelMappingsResolveCreateResult, ModelMappingsUpdateResult, ModelRankingRefreshTriggerRequest, ModelRankingsJobsListResult, ModelRankingsListResult, ModelRankingsRefreshResult, ModelRankingsStatusRetrieveResult, ModelsCreateResult, ModelsDeleteResult, ModelsListResult, ModelsRefreshResult, ModelsUpdateResult, ModelVendorsCreateResult, ModelVendorsListResult } from '../types';
export declare class AiAiResourcesApi {
    private client;
    constructor(client: HttpClient);
    /** List assignable resources */
    list(): Promise<AiResourcesListResult>;
    /** Create ai resource */
    create(body: AdminAiResourceCreateRequest): Promise<AiResourcesCreateResult>;
    /** Update ai resource */
    update(resourceId: string, body: AdminAiResourceUpdateRequest): Promise<AiResourcesUpdateResult>;
}
export declare class AiAiResourceGroupsResourcesApi {
    private client;
    constructor(client: HttpClient);
    /** List resource group resources */
    list(groupIdOrCode: string): Promise<AiResourceGroupsResourcesListResult>;
}
export declare class AiAiResourceGroupsApi {
    private client;
    readonly resources: AiAiResourceGroupsResourcesApi;
    constructor(client: HttpClient);
    /** List resource groups */
    list(): Promise<AiResourceGroupsListResult>;
    /** Create resource group */
    create(body: AdminAiResourceGroupCreateRequest): Promise<AiResourceGroupsCreateResult>;
    /** Delete resource group */
    delete(groupId: string): Promise<AiResourceGroupsDeleteResult>;
    /** Update resource group */
    update(groupId: string, body: AdminAiResourceGroupUpdateRequest): Promise<AiResourceGroupsUpdateResult>;
}
export declare class AiModelsApi {
    private client;
    constructor(client: HttpClient);
    /** List models */
    list(): Promise<ModelsListResult>;
    /** Create model */
    create(body: AdminAiModelCreateRequest): Promise<ModelsCreateResult>;
    /** Sync vendors and models */
    refresh(body: AdminModelCatalogSyncRequest): Promise<ModelsRefreshResult>;
    /** Delete model */
    delete(modelId: string): Promise<ModelsDeleteResult>;
    /** Update model */
    update(modelId: string, body: AdminAiModelUpdateRequest): Promise<ModelsUpdateResult>;
}
export declare class AiModelVendorsApi {
    private client;
    constructor(client: HttpClient);
    /** List vendors */
    list(): Promise<ModelVendorsListResult>;
    /** Create vendor */
    create(body: AdminModelVendorCreateRequest): Promise<ModelVendorsCreateResult>;
}
export interface AiModelRankingsStatusRetrieveParams {
    rankScope?: string;
}
export declare class AiModelRankingsStatusApi {
    private client;
    constructor(client: HttpClient);
    /** List model ranking refresh status */
    retrieve(params?: AiModelRankingsStatusRetrieveParams): Promise<ModelRankingsStatusRetrieveResult>;
}
export interface AiModelRankingsJobsListParams {
    rankScope?: string;
    limit?: string;
}
export declare class AiModelRankingsJobsApi {
    private client;
    constructor(client: HttpClient);
    /** List model ranking refresh jobs */
    list(params?: AiModelRankingsJobsListParams): Promise<ModelRankingsJobsListResult>;
}
export interface AiModelRankingsListParams {
    rankScope?: string;
    vendorCode?: string;
    modality?: string;
    q?: string;
    limit?: string;
}
export declare class AiModelRankingsApi {
    private client;
    readonly jobs: AiModelRankingsJobsApi;
    readonly status: AiModelRankingsStatusApi;
    constructor(client: HttpClient);
    /** List model rankings */
    list(params?: AiModelRankingsListParams): Promise<ModelRankingsListResult>;
    /** Trigger model ranking refresh */
    refresh(body: ModelRankingRefreshTriggerRequest): Promise<ModelRankingsRefreshResult>;
}
export declare class AiModelMappingsResolveApi {
    private client;
    constructor(client: HttpClient);
    /** Resolve model mapping */
    create(body: AdminModelMappingResolveRequest): Promise<ModelMappingsResolveCreateResult>;
}
export interface AiModelMappingsListParams {
    bindingType?: string;
    vendorCode?: string;
    channelId?: string;
    channelCode?: string;
    q?: string;
}
export declare class AiModelMappingsApi {
    private client;
    readonly resolve: AiModelMappingsResolveApi;
    constructor(client: HttpClient);
    /** List model mappings */
    list(params?: AiModelMappingsListParams): Promise<ModelMappingsListResult>;
    /** Create model mapping */
    create(body: AdminModelMappingCreateRequest): Promise<ModelMappingsCreateResult>;
    /** Replace account mappings */
    replace(body: AdminAccountModelMappingsReplaceRequest): Promise<ModelMappingsReplaceResult>;
    /** Delete model mapping */
    delete(mappingId: string): Promise<ModelMappingsDeleteResult>;
    /** Update model mapping */
    update(mappingId: string, body: AdminModelMappingUpdateRequest): Promise<ModelMappingsUpdateResult>;
}
export declare class AiApi {
    private client;
    readonly modelMappings: AiModelMappingsApi;
    readonly modelRankings: AiModelRankingsApi;
    readonly modelVendors: AiModelVendorsApi;
    readonly models: AiModelsApi;
    readonly aiResourceGroups: AiAiResourceGroupsApi;
    readonly aiResources: AiAiResourcesApi;
    constructor(client: HttpClient);
}
export declare function createAiApi(client: HttpClient): AiApi;
//# sourceMappingURL=ai.d.ts.map