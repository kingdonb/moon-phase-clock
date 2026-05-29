// build.gradle.kts snippet for Moon Phase Clock

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.urmanac.moonphase"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.urmanac.moonphase"
        minSdk = 26
        targetSdk = 35
        versionCode = 5
        versionName = "0.0.5"
    }

    buildTypes {
        release {
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
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    
    kotlinOptions {
        jvmTarget = "1.8"
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
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.activity:activity-compose:1.8.2")
    implementation(platform("androidx.compose:compose-bom:2024.02.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.material3:material3")
}
