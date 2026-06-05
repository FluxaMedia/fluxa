# Jetpack Compose
-keepclassmembers class androidx.compose.ui.platform.AndroidComposeView { *; }
-keep class androidx.compose.ui.platform.AndroidComposeView { *; }

# Release log cleanup. Keep warnings/errors for actionable production failures,
# but strip debug/info/verbose calls including their argument evaluation.
-assumenosideeffects class android.util.Log {
    public static boolean isLoggable(java.lang.String, int);
    public static int v(...);
    public static int d(...);
    public static int i(...);
}

# Lifecycle & ViewModel
-keepclassmembers class * extends androidx.lifecycle.ViewModel {
    <init>(...);
}
-keep class * extends androidx.lifecycle.ViewModel { *; }

# Retrofit 2
-keepattributes Signature, InnerClasses, EnclosingMethod
-keepattributes RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations
-keepattributes RuntimeInvisibleAnnotations, RuntimeInvisibleParameterAnnotations
-keepattributes exceptions

-dontwarn retrofit2.**
-keep class retrofit2.** { *; }
-keep interface retrofit2.** { *; }

-keepclasseswithmembers interface * {
    @retrofit2.http.* <methods>;
}

# OkHttp 3
-dontwarn okhttp3.**
-keep class okhttp3.** { *; }
-keep interface okhttp3.** { *; }

# Gson
-keep class com.google.gson.** { *; }
-keep class com.google.gson.reflect.TypeToken
-keep class * extends com.google.gson.reflect.TypeToken
-keepclassmembers class * {
    @com.google.gson.annotations.SerializedName <fields>;
}

# Keep ALL app code — no obfuscation needed and prevents the entire class of
# "add another ProGuard rule" problems caused by R8 renaming our own classes.
# R8 still dead-code-strips unused app code; it just won't rename anything.
-keep class com.fluxa.app.** { *; }
-keep interface com.fluxa.app.** { *; }
-keep class com.fluxa.core.** { *; }
-keep interface com.fluxa.core.** { *; }

# Kotlin Coroutines
-keepnames class kotlinx.coroutines.internal.MainDispatcherLoader
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler
-keepnames class kotlinx.coroutines.android.HandlerContext
-keep class kotlinx.coroutines.** { *; }

# Maintain generic type information for Retrofit
-keep class kotlin.coroutines.Continuation

# TorrServer & Native
-keep class com.fluxa.app.core.rust.** { *; }
-keep class com.fluxa.app.player.TorrServerApi { *; }
-keep class com.fluxa.app.player.TorrStatus { *; }
-keep class com.fluxa.app.player.TorrFileStat { *; }
-keep class com.fluxa.app.player.TorrSettings { *; }

# JNA — native libjnidispatch.so looks up field/method names by string via JNI reflection.
# R8 must not rename or strip anything in com.sun.jna.
-keep class com.sun.jna.** { *; }
-keepclassmembers class com.sun.jna.** { *; }
-dontwarn java.awt.**
-dontwarn com.sun.jna.**

# JNA Structure subclasses — JNA uses reflection to read public fields for memory layout.
# Any class extending Structure anywhere in the app must keep its public fields intact.
-keep class * extends com.sun.jna.Structure { public *; }
-keep class * extends com.sun.jna.Structure$ByReference { public *; }
-keep class * extends com.sun.jna.Structure$ByValue { public *; }
-keep class * extends com.sun.jna.Library { *; }
-keep class * extends com.sun.jna.Callback { *; }

# UniFfi generated bindings — must survive R8 entirely
-keep class com.fluxa.core.uniffi.** { *; }
-keepclassmembers class com.fluxa.core.uniffi.** { *; }

# Coil
-keep class coil.** { *; }

# Room
-keep class * extends androidx.room.RoomDatabase
-keep class * extends androidx.room.Entity
-keep class * extends androidx.room.Dao

# WorkManager + Hilt Workers — keep workers and Hilt-generated AssistedInject factories
-keep class * extends androidx.work.Worker { *; }
-keep class * extends androidx.work.CoroutineWorker { *; }
-keep class * extends androidx.work.ListenableWorker { *; }
-keep class * extends androidx.hilt.work.HiltWorkerFactory { *; }
-keep @androidx.hilt.work.HiltWorker class * { *; }
# Hilt generates *_AssistedFactory and *_HiltModules classes next to each @HiltWorker
-keep class com.fluxa.app.plugins.** { *; }
-keep class **_AssistedFactory { *; }
-keep class **_HiltModules { *; }
-keep class **_HiltModules_* { *; }
-keep @dagger.hilt.InstallIn class * { *; }
-keep @dagger.hilt.android.HiltAndroidApp class * { *; }

# ============================================================
# CloudStream runtime compatibility for external DEX plugins
# Extensions are loaded at runtime as .cs3 DEX files.
# They reference classes by ORIGINAL name, so R8 must NOT
# rename (obfuscate) any class/method they might call.
# ============================================================

# CloudStream library classes - no optimization allowed because R8 marks
# open methods as final, breaking extension inheritance (LinkageError)
-keep class com.lagradost.cloudstream3.** { *; }
-keep interface com.lagradost.cloudstream3.** { *; }

# NiceHttp - extensions call NiceResponse.getDocument() which returns org.jsoup.nodes.Document
-keep class com.lagradost.nicehttp.** { *; }
-keep interface com.lagradost.nicehttp.** { *; }
-dontwarn com.lagradost.nicehttp.**

# CloudStream API logger used by extensions
-keep class com.lagradost.api.** { *; }
-dontwarn com.lagradost.api.**

# Jsoup - MUST keep original class names, R8 was renaming Document to r9.h
# causing NoSuchMethodError for getDocument()Lorg/jsoup/nodes/Document;
-keep class org.jsoup.** { *; }
-keep interface org.jsoup.** { *; }

# OkHttp - referenced in NiceResponse constructor signatures
-keep class okhttp3.** { *; }
-keep interface okhttp3.** { *; }

# Jackson - used by CloudStream extensions for JSON parsing
-keep class com.fasterxml.jackson.** { *; }
-dontwarn com.fasterxml.jackson.**

# Kotlin internals referenced by extensions
-keep class kotlin.** { *; }
-keep class kotlin.coroutines.** { *; }
-keep class kotlin.coroutines.jvm.internal.** { *; }
-keep class kotlin.jvm.internal.** { *; }
-keep class kotlinx.coroutines.** { *; }
-keepattributes Signature,RuntimeVisibleAnnotations,RuntimeInvisibleAnnotations,InnerClasses,EnclosingMethod

# Prefer library classes over local stubs to avoid R8 conflicts
-dontwarn com.lagradost.cloudstream3.**

# Rhino JS engine (used by NewPipe extractor, excluded from dependencies)
-dontwarn org.mozilla.javascript.**

# Google RE2J regex engine (used by Jsoup, optional dependency)
-dontwarn com.google.re2j.**
