/** Admin model mapping rule binding input schema exposed by Claw Router. */
export interface AdminModelMappingRuleBindingInput {
  /** Binding code field on admin model mapping rule binding input. */
  bindingCode?: string | null;
  /** Binding id field on admin model mapping rule binding input. */
  bindingId?: string | null;
  /** Binding name field on admin model mapping rule binding input. */
  bindingName?: string | null;
  /** Binding type field on admin model mapping rule binding input. */
  bindingType: 'global' | 'vendor' | 'channel_group' | 'channel' | 'provider_account' | 'site' | 'site_service';
  /** Enabled field on admin model mapping rule binding input. */
  enabled?: boolean;
  /** Id field on admin model mapping rule binding input. */
  id?: string | null;
}
