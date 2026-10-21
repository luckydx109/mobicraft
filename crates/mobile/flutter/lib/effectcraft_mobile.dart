import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';
import 'package:ffi/ffi.dart';

// Typedefs for C/JNI function signatures
typedef NativeInitC = ffi.Void Function(ffi.Pointer<Utf8> cacheDir, ffi.Pointer<Utf8> internalDir);
typedef NativeInitDart = void Function(ffi.Pointer<Utf8> cacheDir, ffi.Pointer<Utf8> internalDir);

typedef NativeExecuteCmdC = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8> cmd, ffi.Pointer<Utf8> params);
typedef NativeExecuteCmdDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8> cmd, ffi.Pointer<Utf8> params);

typedef NativeSeekC = ffi.Void Function(ffi.Int64 compId, ffi.Int64 tick, ffi.Float scale);
typedef NativeSeekDart = void Function(int compId, int tick, double scale);

typedef NativeProgressC = ffi.Float Function();
typedef NativeProgressDart = double Function();

/// High-level Dart / Flutter client for EffectCraft Mobile.
class EffectCraftMobile {
  static final EffectCraftMobile instance = EffectCraftMobile._();

  late final ffi.DynamicLibrary _dylib;
  late final NativeInitDart _nativeInit;
  late final NativeExecuteCmdDart _nativeExecuteCmd;
  late final NativeSeekDart _nativeSeek;
  late final NativeProgressDart _nativeProgress;

  EffectCraftMobile._() {
    _dylib = Platform.isAndroid
        ? ffi.DynamicLibrary.open('libeffectcraft_mobile.so')
        : ffi.DynamicLibrary.process();

    _nativeInit = _dylib.lookupFunction<NativeInitC, NativeInitDart>('Java_com_effectcraft_EffectCraftBridge_nativeInit');
    _nativeExecuteCmd = _dylib.lookupFunction<NativeExecuteCmdC, NativeExecuteCmdDart>('Java_com_effectcraft_EffectCraftBridge_nativeExecuteCommand');
    _nativeSeek = _dylib.lookupFunction<NativeSeekC, NativeSeekDart>('Java_com_effectcraft_EffectCraftBridge_nativeSeekFrame');
    _nativeProgress = _dylib.lookupFunction<NativeProgressC, NativeProgressDart>('Java_com_effectcraft_EffectCraftBridge_nativeGetExportProgress');
  }

  /// Initialize native engine with local cache folder.
  void init(String cacheDir, String internalDir) {
    final cCache = cacheDir.toNativeUtf8();
    final cInternal = internalDir.toNativeUtf8();
    try {
      _nativeInit(cCache, cInternal);
    } finally {
      malloc.free(cCache);
      malloc.free(cInternal);
    }
  }

  /// Execute an EffectCraft engine command with typed Map parameters.
  Map<String, dynamic> executeCommand(String commandId, Map<String, dynamic> params) {
    final cCmd = commandId.toNativeUtf8();
    final cParams = jsonEncode(params).toNativeUtf8();
    try {
      final resPtr = _nativeExecuteCmd(cCmd, cParams);
      final resStr = resPtr.toDartString();
      return jsonDecode(resStr) as Map<String, dynamic>;
    } finally {
      malloc.free(cCmd);
      malloc.free(cParams);
    }
  }

  /// Seek the timeline to `tick` with specified preview scale (0.33 for 360p, 0.5 for 540p, 1.0 for 1080p).
  void seekFrame({int compId = 0, required int tick, double scale = 0.5}) {
    _nativeSeek(compId, tick, scale);
  }

  /// Current export progress (0.0 to 1.0).
  double getExportProgress() {
    return _nativeProgress();
  }
}
