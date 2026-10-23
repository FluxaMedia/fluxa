-assumenosideeffects class android.util.Log {
    public static boolean isLoggable(java.lang.String, int);
    public static int v(...);
    public static int d(...);
    public static int i(...);
}

-keep class com.fluxa.app.ui.rust.NativeRenderer { *; }
-keep class com.fluxa.app.ui.rust.Mpv { *; }
-keep class com.fluxa.app.ui.rust.EmojiRasterizer { *; }
-keep class com.fluxa.app.ui.rust.NativeInputFocus { *; }
-keep class com.fluxa.app.ui.rust.ArtworkDecoder { *; }
-keep class com.fluxa.app.ui.rust.PlatformHttp { *; }
