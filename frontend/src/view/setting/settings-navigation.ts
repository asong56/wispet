export interface SettingsNavigationItem {
  label: string;
  icon: string;
  to: string;
}

export const settingsNavigation: SettingsNavigationItem[] = [
  { label: '词典设置', icon: 'icon-book', to: '/setting/dict' },
  { label: '通用设置', icon: 'icon-cog',  to: '/setting/software' },
];
