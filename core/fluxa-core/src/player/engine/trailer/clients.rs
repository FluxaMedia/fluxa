use serde_json::{Value, json};

pub(crate) const COUNT: usize = 4;

pub(crate) struct Client {
    pub(crate) id: &'static str,
    pub(crate) version: &'static str,
    pub(crate) user_agent: &'static str,
    pub(crate) context: Value,
}

pub(crate) fn rotated_index(preferred: usize, attempt: usize) -> usize {
    (preferred + attempt) % COUNT
}

pub(crate) fn client(index: usize) -> Client {
    match index % COUNT {
        0 => Client {
            id: "5",
            version: "20.10.4",
            user_agent: "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)",
            context: json!({
                "clientName": "IOS",
                "clientVersion": "20.10.4",
                "deviceMake": "Apple",
                "deviceModel": "iPhone16,2",
                "osName": "iPhone",
                "osVersion": "18.3.2.22D82",
                "hl": "en",
                "gl": "US"
            }),
        },
        1 => Client {
            id: "3",
            version: "20.10.38",
            user_agent: "com.google.android.youtube/20.10.38 (Linux; U; Android 11) gzip",
            context: json!({
                "clientName": "ANDROID",
                "clientVersion": "20.10.38",
                "osName": "Android",
                "osVersion": "11",
                "androidSdkVersion": 30,
                "hl": "en",
                "gl": "US"
            }),
        },
        2 => Client {
            id: "28",
            version: "1.60.19",
            user_agent: "com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip",
            context: json!({
                "clientName": "ANDROID_VR",
                "clientVersion": "1.60.19",
                "deviceMake": "Oculus",
                "deviceModel": "Quest 3",
                "osName": "Android",
                "osVersion": "12L",
                "androidSdkVersion": 32,
                "hl": "en",
                "gl": "US"
            }),
        },
        _ => Client {
            id: "101",
            version: "1.02",
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15",
            context: json!({
                "clientName": "VISIONOS",
                "clientVersion": "1.02",
                "deviceMake": "Apple",
                "deviceModel": "RealityDevice17,1",
                "osName": "visionOS",
                "osVersion": "26.5.23O471",
                "hl": "en",
                "gl": "US"
            }),
        },
    }
}
