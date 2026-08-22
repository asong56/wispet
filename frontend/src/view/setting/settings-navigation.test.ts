import { describe, expect, it } from 'vitest';
import { settingsNavigation } from './settings-navigation';

describe('settingsNavigation', () => {
  it('only contains dict and software settings', () => {
    expect(settingsNavigation.map((item) => item.label)).toEqual(['词典设置', '通用设置']);
  });

  it('all items have label, icon, and unique route', () => {
    expect(settingsNavigation.every((item) => item.label && item.icon && item.to)).toBe(true);
    expect(new Set(settingsNavigation.map((item) => item.to)).size).toBe(settingsNavigation.length);
  });
});
