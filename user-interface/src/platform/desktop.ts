/** Injected by the packaged Windows host before the JS bundle executes. */
type DesktopConfig = { endpoint: string; token: string };
export function desktopConfig(): DesktopConfig | undefined {
  return (globalThis as typeof globalThis & { __AINAVLOG_DESKTOP__?: DesktopConfig }).__AINAVLOG_DESKTOP__;
}
export function deviceEndpoint() {
  return desktopConfig()?.endpoint ?? process.env.EXPO_PUBLIC_DEVICE_INTERFACE_URL?.trim();
}
