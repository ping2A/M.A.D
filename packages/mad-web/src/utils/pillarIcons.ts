export const PILLAR_ICONS: Record<string, string> = {
  cybersecurity_dlp: "🛡️",
  dfir: "🔍",
  platform_os: "📱",
};

export function pillarIcon(id: string): string {
  return PILLAR_ICONS[id] ?? "◆";
}
