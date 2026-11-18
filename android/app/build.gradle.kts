// build.gradle.kts snippet for Moon Phase Clock

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.urmanac.moonphase"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.urmanac.moonphase"
        minSdk = 26
        targetSdk = 36
        versionCode = 10
        versionName = "0.0.10"
    }

    // Release signing is configured only when scripts/release.sh provides the
    // keystore through the environment. Nothing secret is stored in the repo
    // or in Gradle properties; without these variables the release build is
    // simply unsigned.
    val releaseKeystore = System.getenv("MOON_SIGNING_STORE_FILE")
    if (releaseKeystore != null) {
        signingConfigs {
            create("release") {
                storeFile = file(releaseKeystore)
                storePassword = System.getenv("MOON_SIGNING_STORE_PASSWORD")
                keyAlias = System.getenv("MOON_SIGNING_KEY_ALIAS")
                keyPassword = System.getenv("MOON_SIGNING_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            if (releaseKeystore != null) {
                signingConfig = signingConfigs.getByName("release")
            }
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    
    buildFeatures {
        compose = true
    }
    
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    
    kotlin {
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }

    // Unit tests load the same WASM brain that ships in the app, so there is
    // exactly one copy of the binary to keep current.
    sourceSets {
        getByName("test") {
            resources.directories.add("src/main/assets")
        }
    }

    tasks.withType<Test> {
        val libPath = file("native-libs").absolutePath
        systemProperty("java.library.path", libPath)
        systemProperty("jna.library.path", libPath)
    }
}

dependencies {
    // Chicory: Pure Java WebAssembly Interpreter
    // This allows us to run WASM on Android without native .so files.
    implementation("com.dylibso.chicory:runtime:1.0.0-M2")
    implementation("com.dylibso.chicory:wasi:1.0.0-M2")
    
    // Testing
    testImplementation("junit:junit:4.13.2")
    
    // Standard Android/Compose dependencies
    implementation("androidx.core:core-ktx:1.19.1")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation(platform("androidx.compose:compose-bom:2026.09.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.material3:material3")
    
    // WorkManager for background periodic app-icon updates
    implementation("androidx.work:work-runtime-ktx:2.12.0")
}
