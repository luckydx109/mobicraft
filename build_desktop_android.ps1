# EffectCraft - 1-Click Desktop UI on Android Build Script
# This builds the complete, unmodified EffectCraft Desktop UI and packages EffectCraft-Mobile.apk (arm64-v8a).

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  EffectCraft: Building Desktop UI for Android Mobile      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Environment paths configuration
$CargoBin = "$env:USERPROFILE\.cargo\bin"

# Android Home (E: drive Android SDK installed by EffectCraft setup)
$AndroidHome = if (Test-Path "E:\android-tools\android-sdk") { "E:\android-tools\android-sdk" }
               elseif ($env:ANDROID_HOME -and (Test-Path $env:ANDROID_HOME)) { $env:ANDROID_HOME }
               else { "D:\android_studio_files" }

# Java Home (E: drive JDK 17 installed by EffectCraft setup)
$JavaHome = if (Test-Path "E:\android-tools\jdk-17") { "E:\android-tools\jdk-17" }
            elseif ($env:JAVA_HOME -and (Test-Path $env:JAVA_HOME)) { $env:JAVA_HOME }
            else { "D:\android studio\jbr" }

# Android NDK
$AndroidNdk = if (Test-Path "$AndroidHome\ndk\28.0.13004108") { "$AndroidHome\ndk\28.0.13004108" }
              elseif (Test-Path "$AndroidHome\ndk\28.2.13676358") { "$AndroidHome\ndk\28.2.13676358" }
              elseif (Test-Path "$AndroidHome\ndk") { (Get-ChildItem "$AndroidHome\ndk" -Directory | Select-Object -First 1).FullName }
              else { "D:\android_studio_files\ndk\28.2.13676358" }

# Build-tools
$BuildTools = if (Test-Path "$AndroidHome\build-tools\35.0.0") { "$AndroidHome\build-tools\35.0.0" }
              elseif (Test-Path "$AndroidHome\build-tools") { (Get-ChildItem "$AndroidHome\build-tools" -Directory | Sort-Object Name -Descending | Select-Object -First 1).FullName }
              else { "$AndroidHome\build-tools\35.0.0" }

# Platform Jar (check 36, 35, 34)
$PlatformJar = if (Test-Path "$AndroidHome\platforms\android-36\android.jar") { "$AndroidHome\platforms\android-36\android.jar" }
               elseif (Test-Path "$AndroidHome\platforms\android-35\android.jar") { "$AndroidHome\platforms\android-35\android.jar" }
               elseif (Test-Path "$AndroidHome\platforms\android-34\android.jar") { "$AndroidHome\platforms\android-34\android.jar" }
               else { "$AndroidHome\platforms\android-35\android.jar" }

$PlatformTools = "$AndroidHome\platform-tools"
$NdkBin = "$AndroidNdk\toolchains\llvm\prebuilt\windows-x86_64\bin"

Write-Host "Using Android SDK: $AndroidHome" -ForegroundColor Gray
Write-Host "Using Java Home:   $JavaHome" -ForegroundColor Gray
Write-Host "Using NDK:         $AndroidNdk" -ForegroundColor Gray
Write-Host "Using Platform:    $PlatformJar" -ForegroundColor Gray

if (Test-Path $CargoBin) {
    $env:PATH = "$CargoBin;$env:PATH"
}
if (Test-Path "$JavaHome\bin") {
    $env:JAVA_HOME = $JavaHome
    $env:PATH = "$JavaHome\bin;$env:PATH"
}
if (Test-Path $NdkBin) {
    $env:PATH = "$NdkBin;$env:PATH"
}
if (Test-Path $BuildTools) {
    $env:PATH = "$BuildTools;$env:PATH"
}
if (Test-Path $PlatformTools) {
    $env:PATH = "$PlatformTools;$env:PATH"
}

$env:ANDROID_HOME = $AndroidHome
$env:ANDROID_NDK_HOME = $AndroidNdk

# cargo links with the NDK's clang for the phone (these override .cargo/config.toml)
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "$NdkBin\aarch64-linux-android26-clang.cmd"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_AR = "$NdkBin\llvm-ar.exe"

# 2. Check Rust & Target
Write-Host "[1/5] Checking Rust compiler and Android target..." -ForegroundColor Yellow
if (-not (Get-Command "cargo.exe" -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo not found in PATH ($CargoBin\cargo.exe)."
}

& rustup target add aarch64-linux-android | Out-Null

# 3. Compile Desktop App as Android Native Library (.so)
Write-Host "[2/5] Compiling Desktop UI for Android aarch64 (libeffectcraft.so)..." -ForegroundColor Yellow
$WorkspaceRoot = $PSScriptRoot
Set-Location $WorkspaceRoot

# Only the library (the desktop executable is not built for Android) in the quick `android` profile
# (see Cargo.toml): no LTO, parallel codegen, incremental. Keep the target folder between builds.
& cargo build --package effectcraft --lib --target aarch64-linux-android --profile android
if ($LASTEXITCODE -ne 0) { Write-Error "cargo build failed" }

$SoSource = "$WorkspaceRoot\target\aarch64-linux-android\android\libeffectcraft.so"
$JniTargetDir = "$WorkspaceRoot\apps\effectcraft-android\app\src\main\jniLibs\arm64-v8a"

if (-not (Test-Path $SoSource)) {
    Write-Error "Build finished but libeffectcraft.so was not found at $SoSource"
}

# 4. Copy shared library to Android wrapper
Write-Host "[3/5] Placing libeffectcraft.so into Android app package..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $JniTargetDir | Out-Null
Copy-Item -Path $SoSource -Destination "$JniTargetDir\libeffectcraft.so" -Force
Write-Host "  -> Successfully deployed libeffectcraft.so (arm64-v8a)" -ForegroundColor Green

# 5. Package Direct Installable APK
Write-Host "[4/5] Packaging EffectCraft-Mobile.apk directly..." -ForegroundColor Yellow
$BuildDir = "$WorkspaceRoot\target\android_apk_build"
New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null

$ResDir = "$WorkspaceRoot\apps\effectcraft-android\app\src\main\res"
$Manifest = "$WorkspaceRoot\apps\effectcraft-android\app\src\main\AndroidManifest.xml"
$ResZip = "$BuildDir\res.zip"
$UnsignedApk = "$BuildDir\unsigned.apk"
$AlignedApk = "$BuildDir\aligned.apk"
$FinalApk = "$WorkspaceRoot\EffectCraft-Mobile.apk"

# Java: MainActivity (the gallery / file manager pickers) -> classes.dex
$JavaSrc = "$WorkspaceRoot\apps\effectcraft-android\app\src\main\java"
$ClassesDir = "$BuildDir\classes"
$DexDir = "$BuildDir\dex"
if (Test-Path $ClassesDir) { Remove-Item $ClassesDir -Recurse -Force }
if (Test-Path $DexDir) { Remove-Item $DexDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path $ClassesDir, $DexDir | Out-Null
$JavaFiles = (Get-ChildItem $JavaSrc -Recurse -Filter *.java).FullName
& "$JavaHome\bin\javac.exe" -source 8 -target 8 -bootclasspath $PlatformJar -d $ClassesDir $JavaFiles
if ($LASTEXITCODE -ne 0) { Write-Error "javac failed" }
$ClassFiles = (Get-ChildItem $ClassesDir -Recurse -Filter *.class).FullName
& "$BuildTools\d8.bat" --lib $PlatformJar --min-api 26 --output $DexDir $ClassFiles
if ($LASTEXITCODE -ne 0) { Write-Error "d8 failed" }

# Compile resources
& "$BuildTools\aapt2.exe" compile --dir $ResDir -o $ResZip
# Link APK with explicit SDK versions and versioning
& "$BuildTools\aapt2.exe" link -I $PlatformJar --manifest $Manifest --min-sdk-version 26 --target-sdk-version 34 --version-code 1 --version-name "1.0.0" -o $UnsignedApk $ResZip

# Inject native library into APK zip (lib/arm64-v8a/libeffectcraft.so)
[Reflection.Assembly]::LoadWithPartialName('System.IO.Compression') | Out-Null
[Reflection.Assembly]::LoadWithPartialName('System.IO.Compression.FileSystem') | Out-Null
$zip = [System.IO.Compression.ZipFile]::Open($UnsignedApk, [System.IO.Compression.ZipArchiveMode]::Update)
$existing = $zip.GetEntry("lib/arm64-v8a/libeffectcraft.so")
if ($existing) { $existing.Delete() }
$entry = $zip.CreateEntry("lib/arm64-v8a/libeffectcraft.so", [System.IO.Compression.CompressionLevel]::Optimal)
$writer = $entry.Open()
$reader = [System.IO.File]::OpenRead("$JniTargetDir\libeffectcraft.so")
$reader.CopyTo($writer)
$reader.Dispose()
$writer.Dispose()
$existingDex = $zip.GetEntry("classes.dex")
if ($existingDex) { $existingDex.Delete() }
$dexEntry = $zip.CreateEntry("classes.dex", [System.IO.Compression.CompressionLevel]::Optimal)
$dexWriter = $dexEntry.Open()
$dexReader = [System.IO.File]::OpenRead("$DexDir\classes.dex")
$dexReader.CopyTo($dexWriter)
$dexReader.Dispose()
$dexWriter.Dispose()
$zip.Dispose()

# Zipalign
if (Test-Path $AlignedApk) { Remove-Item $AlignedApk -Force }
& "$BuildTools\zipalign.exe" -v -p 4 $UnsignedApk $AlignedApk | Out-Null

# Debug Keystore creation if needed
$Keystore = "$BuildDir\debug.keystore"
if (-not (Test-Path $Keystore)) {
    & "$JavaHome\bin\keytool.exe" -genkey -v -keystore $Keystore -storepass android -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US" | Out-Null
}

# Sign APK
& "$BuildTools\apksigner.bat" sign --ks $Keystore --ks-pass pass:android --key-pass pass:android --out $FinalApk $AlignedApk

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  SUCCESS! APK Generated: $FinalApk" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green

# 6. Check ADB Device
Write-Host "[5/5] Checking connected Android device..." -ForegroundColor Yellow
if (Get-Command "adb.exe" -ErrorAction SilentlyContinue) {
    $devices = & adb devices | Where-Object { $_ -match "\tdevice$" }
    if ($devices) {
        Write-Host "Connected phone found! Installing EffectCraft-Mobile.apk..." -ForegroundColor Cyan
        & adb install -r $FinalApk
        Write-Host "App installed! Launching on phone..." -ForegroundColor Green
        & adb shell am start -n ai.storyteller.effectcraft/.MainActivity
    } else {
        Write-Host "No phone connected via USB. You can copy '$FinalApk' to your phone and install it!" -ForegroundColor White
    }
}
