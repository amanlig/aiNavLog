Pod::Spec.new do |s|
  s.name = 'AiNavLog'
  s.version = '0.1.0'
  s.summary = 'aiNavLog Rust btleplug mobile bridge'
  s.description = s.summary
  s.license = { :type => 'Proprietary' }
  s.author = 'Kode Komplete Software Services Ltd.'
  s.homepage = 'https://github.com/amanlig/aiNavLog'
  s.source = { :git => 'https://github.com/amanlig/aiNavLog.git' }
  s.platform = :ios, '16.4'
  s.swift_version = '5.9'
  s.static_framework = true
  s.dependency 'ExpoModulesCore'
  s.source_files = '*.swift', 'include/*.h'
  s.public_header_files = 'include/ainavlog.h'
  s.vendored_frameworks = 'builds/AiNavLogBridge.xcframework'
  s.resources = 'builds/notices/NativeNotices.txt'
  s.frameworks = 'CoreBluetooth', 'CoreFoundation', 'Foundation', 'Security'
  s.libraries = 'c++', 'iconv', 'resolv', 'objc'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES' }
end
