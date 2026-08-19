import { coreInvoke } from './engineCoreClient';
import type { Stream, StreamBadge, StreamBadgeImport, StreamBadgeRules } from './types';

export async function coreParseStreamBadgeImport(sourceUrl: string, payload: string): Promise<StreamBadgeImport | null> {
  return coreInvoke<StreamBadgeImport>('parseStreamBadgeImport', JSON.stringify({ sourceUrl, payload }));
}

export async function coreNormalizeStreamBadgeRules(rules: StreamBadgeRules): Promise<StreamBadgeRules | null> {
  return coreInvoke<StreamBadgeRules>('normalizeStreamBadgeRules', JSON.stringify({ rulesJson: JSON.stringify(rules) }));
}

export async function coreUpsertStreamBadgeImport(
  rules: StreamBadgeRules,
  streamBadgeImport: StreamBadgeImport,
  activate = true,
): Promise<StreamBadgeRules | null> {
  return coreInvoke<StreamBadgeRules>(
    'upsertStreamBadgeImport',
    JSON.stringify({ rulesJson: JSON.stringify(rules), importJson: JSON.stringify(streamBadgeImport), activate }),
  );
}

export async function coreSetActiveStreamBadgeSource(rules: StreamBadgeRules, sourceUrl: string): Promise<StreamBadgeRules | null> {
  return coreInvoke<StreamBadgeRules>('setActiveStreamBadgeSource', JSON.stringify({ rulesJson: JSON.stringify(rules), sourceUrl }));
}

export async function coreRemoveStreamBadgeSource(rules: StreamBadgeRules, sourceUrl: string): Promise<StreamBadgeRules | null> {
  return coreInvoke<StreamBadgeRules>('removeStreamBadgeSource', JSON.stringify({ rulesJson: JSON.stringify(rules), sourceUrl }));
}

export async function coreMatchStreamBadges(stream: Stream, rules: StreamBadgeRules): Promise<StreamBadge[]> {
  return (
    (await coreInvoke<StreamBadge[]>(
      'matchStreamBadges',
      JSON.stringify({ streamJson: JSON.stringify(stream), rulesJson: JSON.stringify(rules) }),
    )) ?? []
  );
}
