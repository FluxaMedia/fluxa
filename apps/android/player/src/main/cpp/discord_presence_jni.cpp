#define DISCORDPP_IMPLEMENTATION
#include <discordpp.h>

#include <jni.h>
#include <atomic>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <chrono>
#include <thread>

namespace {

std::mutex client_mutex;
std::shared_ptr<discordpp::Client> client;
std::atomic_bool callback_loop_started = false;
bool authorization_started = false;
std::optional<discordpp::Activity> pending_activity;

std::optional<std::string> string_from_jni(JNIEnv* env, jstring value) {
    if (value == nullptr) return std::nullopt;
    const char* chars = env->GetStringUTFChars(value, nullptr);
    if (chars == nullptr) return std::nullopt;
    std::string result(chars);
    env->ReleaseStringUTFChars(value, chars);
    return result;
}

}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_fluxa_app_player_DiscordPresenceNative_initializeNative(JNIEnv*, jclass, jlong application_id) {
    std::lock_guard lock(client_mutex);
    if (client != nullptr) return JNI_TRUE;
    client = std::make_shared<discordpp::Client>();
    client->SetApplicationId(static_cast<uint64_t>(application_id));
    client->SetStatusChangedCallback([](discordpp::Client::Status status, discordpp::Client::Error, int32_t) {
        if (status != discordpp::Client::Status::Ready) return;
        std::shared_ptr<discordpp::Client> current_client;
        std::optional<discordpp::Activity> activity;
        {
            std::lock_guard lock(client_mutex);
            current_client = client;
            if (pending_activity.has_value()) {
                activity = std::move(*pending_activity);
                pending_activity.reset();
            }
        }
        if (current_client != nullptr && activity.has_value()) {
            current_client->UpdateRichPresence(std::move(*activity), [](discordpp::ClientResult) {});
        }
    });
    if (!callback_loop_started) {
        callback_loop_started = true;
        std::thread([] {
            while (callback_loop_started) {
                discordpp::RunCallbacks();
                std::this_thread::sleep_for(std::chrono::milliseconds(10));
            }
        }).detach();
    }
    return JNI_TRUE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_fluxa_app_player_DiscordPresenceNative_authorizeNative(JNIEnv*, jclass) {
    std::lock_guard lock(client_mutex);
    if (client == nullptr || authorization_started ||
        client->GetStatus() == discordpp::Client::Status::Ready) return JNI_FALSE;
    authorization_started = true;
    auto current_client = client;
    auto verifier = current_client->CreateAuthorizationCodeVerifier();
    discordpp::AuthorizationArgs args;
    args.SetClientId(1518004842860122174);
    args.SetScopes(discordpp::Client::GetDefaultPresenceScopes());
    args.SetCodeChallenge(verifier.Challenge());
    current_client->Authorize(std::move(args), [current_client, verifier](auto result, auto code, auto redirect_uri) {
        if (!result.Successful()) {
            std::lock_guard lock(client_mutex);
            authorization_started = false;
            return;
        }
        current_client->GetToken(1518004842860122174, code, verifier.Verifier(), redirect_uri,
            [current_client](auto token_result, auto access_token, auto, auto token_type, auto, auto) {
                if (!token_result.Successful()) {
                    std::lock_guard lock(client_mutex);
                    authorization_started = false;
                    return;
                }
                current_client->UpdateToken(token_type, access_token, [current_client](auto update_result) {
                    if (update_result.Successful()) {
                        current_client->Connect();
                    } else {
                        std::lock_guard lock(client_mutex);
                        authorization_started = false;
                    }
                });
            });
    });
    return JNI_TRUE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_fluxa_app_player_DiscordPresenceNative_updateNative(
    JNIEnv* env,
    jclass,
    jstring title,
    jstring episode_line,
    jstring status,
    jlong position_ms,
    jlong duration_ms,
    jstring artwork_url) {
    std::lock_guard lock(client_mutex);
    if (client == nullptr) return JNI_FALSE;

    discordpp::Activity activity;
    activity.SetType(discordpp::ActivityTypes::Watching);
    activity.SetDetails(string_from_jni(env, title));
    activity.SetState(string_from_jni(env, episode_line));
    activity.SetStatusDisplayType(discordpp::StatusDisplayTypes::Details);
    discordpp::ActivityAssets assets;
    assets.SetLargeImage(string_from_jni(env, artwork_url));
    assets.SetLargeText(string_from_jni(env, title));
    activity.SetAssets(assets);
    const auto status_text = string_from_jni(env, status).value_or("Paused");
    if (status_text == "Watching" && position_ms >= 0 && duration_ms > position_ms) {
        discordpp::ActivityTimestamps timestamps;
        const auto now = std::chrono::duration_cast<std::chrono::seconds>(
            std::chrono::system_clock::now().time_since_epoch()).count();
        timestamps.SetStart(static_cast<uint64_t>(now - position_ms / 1000));
        timestamps.SetEnd(static_cast<uint64_t>(now + (duration_ms - position_ms) / 1000));
        activity.SetTimestamps(timestamps);
    }
    if (client->GetStatus() == discordpp::Client::Status::Ready) {
        client->UpdateRichPresence(std::move(activity), [](discordpp::ClientResult) {});
    } else {
        pending_activity = std::move(activity);
    }
    return JNI_TRUE;
}

extern "C" JNIEXPORT void JNICALL
Java_com_fluxa_app_player_DiscordPresenceNative_clearNative(JNIEnv*, jclass) {
    std::lock_guard lock(client_mutex);
    if (client == nullptr) return;
    pending_activity.reset();
    client->UpdateRichPresence(discordpp::Activity(), [](discordpp::ClientResult) {});
}
