/** Admin model mapping rule binding schema exposed by Claw Router. */
export interface AdminModelMappingRuleBinding {
  /** Binding code field on admin model mapping rule binding. */
  bindingCode?: string | null;
  /** Binding id field on admin model mapping rule binding. */
  bindingId?: string | null;
  /** Binding name field on admin model mapping rule binding. */
  bindingName?: string | null;
  /** Binding type field on admin model mapping rule binding. */
  bindingType: 'global' | 'vendor' | 'channel_group' | 'channel' | 'provider_account' | 'site' | 'site_service';
  /** Created at field on admin model mapping rule binding. */
  createdAt?: string | null;
  /** Enabled field on admin model mapping rule binding. */
  enabled: boolean;
  /** Id field on admin model mapping rule binding. */
  id: string;
  /** Sort order field on admin model mapping rule binding. */
  sortOrder: string;
  /** Updated at field on admin model mapping rule binding. */
  updatedAt?: string | null;
}
