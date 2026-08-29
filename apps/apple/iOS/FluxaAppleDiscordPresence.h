#import <stdint.h>

void FluxaDiscordPresenceInitialize(uint64_t applicationId);
void FluxaDiscordPresenceAuthorize(void);
void FluxaDiscordPresenceUpdate(const char *title, const char *episodeLine, const char *status, int64_t positionMs, int64_t durationMs, const char *artworkUrl);
void FluxaDiscordPresenceClear(void);
