import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

// The release key lives beside the project and is never committed (`keystore.properties` is
// ignored), so builds on this machine are signed while the repo stays free of it.
val keystoreProperties = Properties().apply {
    val file = rootProject.file("keystore.properties")
    if (file.exists()) file.inputStream().use { load(it) }
}

// The emulator is a Rust cdylib produced by cargo-ndk. arm64-v8a remains the default;
// pass -PtargetAbi=armeabi-v7a to package the 32-bit ARM build instead.
val targetAbi = providers.gradleProperty("targetAbi").orElse("arm64-v8a").get()
val rustTarget = when (targetAbi) {
    "arm64-v8a" -> "aarch64-linux-android"
    "armeabi-v7a" -> "armv7-linux-androideabi"
    else -> error("ABI no soportada: $targetAbi")
}
val emulatorSo = rootProject.file("../src/OpenGMK/target/$rustTarget/release/libgm8emulator.so")
val jniLibDir = layout.projectDirectory.dir("src/main/jniLibs/$targetAbi")
val copyEmulatorSo by tasks.registering {
    inputs.file(emulatorSo).withPropertyName("emulatorSo").optional()
    outputs.dir(jniLibDir)
    doLast {
        if (!emulatorSo.isFile) error("falta ${emulatorSo}; compila con: cargo ndk -t $targetAbi --platform 29 build -p gm8emulator --release")
        val magic = emulatorSo.inputStream().use { it.readNBytes(4) }
        if (magic.size < 4 || magic[0] != 0x7F.toByte() || magic[1] != 'E'.code.toByte() || magic[2] != 'L'.code.toByte() || magic[3] != 'F'.code.toByte()) {
            error("${emulatorSo} no es un ELF valido (¿quedó a cero tras un corte?)")
        }
        val out = jniLibDir.asFile.apply { mkdirs() }.resolve("libgm8emulator.so")
        emulatorSo.copyTo(out, overwrite = true)
    }
}

android {
    namespace = "com.klisee.reloaded"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.klisee.reloaded"
        minSdk = 29
        targetSdk = 36
        versionCode = 10
        versionName = "0.3.1"
        ndk { abiFilters += targetAbi }
    }

    signingConfigs {
        create("release") {
            if (!keystoreProperties.isEmpty) {
                storeFile = rootProject.file(keystoreProperties.getProperty("storeFile"))
                storePassword = keystoreProperties.getProperty("storePassword")
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            // Signed with the project's own key when it is present, so the APK shipped as a release
            // is a real release build and not a debug one.
            signingConfig = if (!keystoreProperties.isEmpty) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures { compose = true }

    packaging { jniLibs { useLegacyPackaging = false } }
}

tasks.named("preBuild") { dependsOn(copyEmulatorSo) }

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.activity:activity-compose:1.10.1")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.compose.ui:ui:1.7.8")
    implementation("androidx.compose.ui:ui-graphics:1.7.8")
    implementation("androidx.compose.foundation:foundation:1.7.8")
    implementation("androidx.compose.runtime:runtime:1.7.8")
    implementation("androidx.compose.material3:material3:1.3.2")
    implementation("androidx.compose.material:material-icons-extended:1.7.8")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
}
