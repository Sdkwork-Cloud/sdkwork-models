/** One model exposed by a model access channel. */
export interface AppModelAccessChannelModel {
  catalogKey: string;
  model: string;
  displayName: string;
  contextTokens?: number | null;
  maxOutputTokens?: number | null;
  toolCallRounds?: number | null;
  supportsMultimodal?: boolean | null;
}
