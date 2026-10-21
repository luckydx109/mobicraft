plugins {
    id("com.android.application")
}

android {
    namespace = "ai.storyteller.effectcraft"
    compileSdk = 35

    defaultConfig {
        applicationId = "ai.storyteller.effectcraft"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.7.0"

        ndk {
            abiFilters.addAll(listOf("arm64-v8a"))
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    // Pure NativeActivity requires no JVM dependencies
}
