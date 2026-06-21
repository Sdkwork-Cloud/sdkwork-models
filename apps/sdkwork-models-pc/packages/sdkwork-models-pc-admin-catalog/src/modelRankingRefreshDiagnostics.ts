import type { ModelRankingRefreshStatusView } from './modelService';

type ModelRankingRefreshJobView = NonNullable<ModelRankingRefreshStatusView['latestJob']>;
type ModelRankingRefreshJobStatus = ModelRankingRefreshJobView['status'];

export type ModelRankingRefreshHealthTone = 'healthy' | 'warning' | 'critical' | 'neutral';

export interface ModelRankingRefreshJobDiagnostic {
  id: string;
  status: ModelRankingRefreshJobStatus;
  statusLabel: string;
  startedAtLabel: string;
  endedAtLabel: string;
  durationLabel: string;
  generatedSummary: string;
  windowLabel: string;
  failureReason: string | null;
}

export interface ModelRankingRefreshDiagnostics {
  status: ModelRankingRefreshStatusView['status'];
  statusLabel: string;
  healthTone: ModelRankingRefreshHealthTone;
  rankScope: string;
  snapshotLabel: string;
  generatedSummary: string;
  refreshSchedule: string;
  windowLabel: string;
  generatedAtLabel: string;
  sourceTablesLabel: string;
  latestJob: ModelRankingRefreshJobDiagnostic | null;
}

export function deriveModelRankingRefreshDiagnostics(
  status: ModelRankingRefreshStatusView,
): ModelRankingRefreshDiagnostics {
  const latestJob = status.latestJob ? deriveJobDiagnostic(status.latestJob) : null;
  return {
    status: status.status,
    statusLabel: labelStatus(status.status),
    healthTone: deriveHealthTone(status.status, latestJob?.status),
    rankScope: status.rankScope,
    snapshotLabel: joinAvailable([status.snapshotDate, status.snapshotPeriod], ' / ') || 'Snapshot unavailable',
    generatedSummary: rankingSummary(status.generatedCount, status.sourceCount),
    refreshSchedule: `Every ${formatInterval(status.refreshIntervalSeconds)}; next ${formatInstant(status.nextRefreshAt)}`,
    windowLabel: windowLabel(status.windowStart, status.windowEnd),
    generatedAtLabel: formatInstant(status.generatedAt),
    sourceTablesLabel: status.sourceTables.length > 0 ? status.sourceTables.join(', ') : 'Source tables unavailable',
    latestJob,
  };
}

function deriveJobDiagnostic(job: ModelRankingRefreshJobView): ModelRankingRefreshJobDiagnostic {
  return {
    id: job.id,
    status: job.status,
    statusLabel: labelStatus(job.status),
    startedAtLabel: formatInstant(job.startedAt),
    endedAtLabel: formatInstant(job.endedAt),
    durationLabel: formatDuration(job.durationMs),
    generatedSummary: rankingSummary(job.generatedCount, job.sourceCount),
    windowLabel: windowLabel(job.windowStart, job.windowEnd),
    failureReason: job.failureReason,
  };
}

function deriveHealthTone(
  status: ModelRankingRefreshStatusView['status'],
  latestJobStatus?: ModelRankingRefreshJobStatus,
): ModelRankingRefreshHealthTone {
  if (latestJobStatus === 'failed') {
    return 'critical';
  }
  if (latestJobStatus === 'running') {
    return 'neutral';
  }
  if (status === 'ready') {
    return 'healthy';
  }
  if (status === 'empty') {
    return 'warning';
  }
  return 'critical';
}

function labelStatus(status: ModelRankingRefreshStatusView['status'] | ModelRankingRefreshJobStatus): string {
  switch (status) {
    case 'ready':
      return 'Ready';
    case 'empty':
      return 'Empty';
    case 'unavailable':
      return 'Unavailable';
    case 'succeeded':
      return 'Succeeded';
    case 'failed':
      return 'Failed';
    case 'skipped':
      return 'Skipped';
    case 'running':
      return 'Running';
  }
  return 'Unknown';
}

function rankingSummary(generatedCount: number, sourceCount: number): string {
  return `${formatInteger(generatedCount)} ranking rows / ${formatInteger(sourceCount)} source rows`;
}

function windowLabel(start: string, end: string): string {
  const startLabel = formatInstant(start);
  const endLabel = formatInstant(end);
  if (startLabel === 'unavailable' || endLabel === 'unavailable') {
    return 'Window unavailable';
  }
  return `${startLabel} -> ${endLabel}`;
}

function formatInstant(value: string): string {
  const normalized = value.trim();
  if (!normalized) {
    return 'unavailable';
  }
  const instant = new Date(normalizeIsoInstant(normalized));
  if (!Number.isFinite(instant.getTime())) {
    return normalized;
  }
  return instant.toISOString().replace(/\.\d{3}Z$/, ' UTC').replace('T', ' ');
}

function normalizeIsoInstant(value: string): string {
  if (!value.includes('T')) {
    return value;
  }
  if (/[zZ]$/.test(value) || /[+-]\d{2}:?\d{2}$/.test(value)) {
    return value;
  }
  return `${value}Z`;
}

function formatInterval(seconds: number): string {
  const normalized = Math.max(0, Math.floor(seconds));
  if (normalized === 0) {
    return '0s';
  }
  if (normalized % 86400 === 0) {
    return `${normalized / 86400}d`;
  }
  if (normalized % 3600 === 0) {
    return `${normalized / 3600}h`;
  }
  if (normalized % 60 === 0) {
    return `${normalized / 60}m`;
  }
  return `${normalized}s`;
}

function formatDuration(milliseconds: number): string {
  const normalized = Math.max(0, Math.floor(milliseconds));
  if (normalized < 1000) {
    return `${normalized}ms`;
  }
  const seconds = normalized / 1000;
  if (seconds < 60) {
    return `${formatCompactNumber(seconds)}s`;
  }
  const minutes = seconds / 60;
  if (minutes < 60) {
    return `${formatCompactNumber(minutes)}m`;
  }
  return `${formatCompactNumber(minutes / 60)}h`;
}

function formatCompactNumber(value: number): string {
  return Number(value.toFixed(1)).toLocaleString(undefined, { maximumFractionDigits: 1 });
}

function formatInteger(value: number): string {
  return Math.max(0, Math.floor(value)).toLocaleString();
}

function joinAvailable(values: readonly string[], separator: string): string {
  return values.map((value) => value.trim()).filter(Boolean).join(separator);
}
