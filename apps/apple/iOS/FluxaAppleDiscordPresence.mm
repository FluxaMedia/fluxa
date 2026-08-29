#define DISCORDPP_IMPLEMENTATION
#include <discordpp.h>

#include <chrono>
#include <atomic>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <thread>

namespace {

std::mutex presenceMutex;
std::shared_ptr<discordpp::Client> presenceClient;
std::optional<discordpp::Activity> pendingActivity;
bool authorizationStarted = false;
std::atomic_bool callbackLoopStarted = false;

std::optional<std::string> optionalString(const char *value) {
    if (value == nullptr || *value == '\0') return std::nullopt;
    return std::string(value);
}

}

void FluxaDiscordPresenceInitialize(uint64_t applicationId) {
    std::lock_guard lock(presenceMutex);
    if (presenceClient != nullptr) return;
    presenceClient = std::make_shared<discordpp::Client>();
    presenceClient->SetApplicationId(applicationId);
    if (!callbackLoopStarted) {
        callbackLoopStarted = true;
        std::thread([] {
            while (callbackLoopStarted) {
                discordpp::RunCallbacks();
                std::this_thread::sleep_for(std::chrono::milliseconds(10));
            }
        }).detach();
    }
    presenceClient->SetStatusChangedCallback([](discordpp::Client::Status status, discordpp::Client::Error, int32_t) {
        if (status != discordpp::Client::Status::Ready) return;
        std::shared_ptr<discordpp::Client> currentClient;
        std::optional<discordpp::Activity> activity;
        {
            std::lock_guard lock(presenceMutex);
            currentClient = presenceClient;
            if (currentClient != nullptr && pendingActivity.has_value()) {
                activity = std::move(*pendingActivity);
                pendingActivity.reset();
            }
        }
        if (currentClient != nullptr && activity.has_value()) {
            currentClient->UpdateRichPresence(std::move(*activity), [](discordpp::ClientResult) {});
        }
    });
}

void FluxaDiscordPresenceAuthorize(void) {
    std::lock_guard lock(presenceMutex);
    if (presenceClient == nullptr || authorizationStarted ||
        presenceClient->GetStatus() == discordpp::Client::Status::Ready) return;
    authorizationStarted = true;
    auto verifier = presenceClient->CreateAuthorizationCodeVerifier();
    discordpp::AuthorizationArgs args;
    args.SetClientId(1518004842860122174);
    args.SetScopes(discordpp::Client::GetDefaultPresenceScopes());
    args.SetCodeChallenge(verifier.Challenge());
    auto currentClient = presenceClient;
    currentClient->Authorize(std::move(args), [currentClient, verifier](auto result, auto code, auto redirectUri) {
        if (!result.Successful()) return;
        currentClient->GetToken(1518004842860122174, code, verifier.Verifier(), redirectUri,
            [currentClient](auto tokenResult, auto accessToken, auto, auto tokenType, auto, auto) {
                if (!tokenResult.Successful()) return;
                currentClient->UpdateToken(tokenType, accessToken, [currentClient](auto updateResult) {
                    if (updateResult.Successful()) currentClient->Connect();
                });
            });
    });
}

void FluxaDiscordPresenceUpdate(
    const char *title,
    const char *episodeLine,
    const char *status,
    int64_t positionMs,
    int64_t durationMs,
    const char *artworkUrl) {
    std::shared_ptr<discordpp::Client> currentClient;
    {
        std::lock_guard lock(presenceMutex);
        currentClient = presenceClient;
    }
    if (currentClient == nullptr) return;
    discordpp::Activity activity;
    activity.SetType(discordpp::ActivityTypes::Watching);
    activity.SetDetails(optionalString(title));
    std::string state = optionalString(episodeLine).value_or("");
    if (const auto statusValue = optionalString(status); statusValue.has_value()) {
        if (!state.empty()) state += " · ";
        state += *statusValue;
    }
    activity.SetState(optionalString(state.c_str()));
    activity.SetStatusDisplayType(discordpp::StatusDisplayTypes::Details);
    discordpp::ActivityAssets assets;
    assets.SetLargeImage(optionalString(artworkUrl));
    assets.SetLargeText(optionalString(title));
    activity.SetAssets(assets);
    const auto statusValue = optionalString(status).value_or("Paused");
    if (statusValue == "Watching" && positionMs >= 0 && durationMs > positionMs) {
        const auto now = std::chrono::duration_cast<std::chrono::seconds>(
            std::chrono::system_clock::now().time_since_epoch()).count();
        discordpp::ActivityTimestamps timestamps;
        timestamps.SetStart(static_cast<uint64_t>(now - positionMs / 1000));
        timestamps.SetEnd(static_cast<uint64_t>(now + (durationMs - positionMs) / 1000));
        activity.SetTimestamps(timestamps);
    }
    bool ready = false;
    {
        std::lock_guard lock(presenceMutex);
        currentClient = presenceClient;
        ready = currentClient != nullptr && currentClient->GetStatus() == discordpp::Client::Status::Ready;
        if (!ready) pendingActivity = std::move(activity);
    }
    if (ready) currentClient->UpdateRichPresence(std::move(activity), [](discordpp::ClientResult) {});
}

void FluxaDiscordPresenceClear(void) {
    std::lock_guard lock(presenceMutex);
    if (presenceClient == nullptr) return;
    pendingActivity.reset();
    presenceClient->UpdateRichPresence(discordpp::Activity(), [](discordpp::ClientResult) {});
}
