const { withInfoPlist, withGradleProperties } = require('expo/config-plugins');
module.exports = function withNativeBmv(config) {
  config = withInfoPlist(config, config => {
    config.modResults.NSBluetoothAlwaysUsageDescription = 'aiNavLog uses Bluetooth to receive nearby battery monitor and simulator readings.';
    return config;
  });
  return withGradleProperties(config, config => {
    config.modResults = config.modResults.filter(item => item.key !== 'reactNativeArchitectures');
    config.modResults.push({ type: 'property', key: 'reactNativeArchitectures', value: 'arm64-v8a,x86_64' });
    return config;
  });
};
