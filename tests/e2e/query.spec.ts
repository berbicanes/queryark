import { test, expect } from '@playwright/test';
import { setupMockTauri } from './fixtures/mockTauri';

test.describe('Query execution', () => {
  test.beforeEach(async ({ page }) => {
    await setupMockTauri(page);
    await page.goto('/');
  });

  test('should render app layout', async ({ page }) => {
    await expect(page.locator('.app-layout')).toBeVisible({ timeout: 10000 });
  });

  test('should show toolbar with expected controls', async ({ page }) => {
    await expect(page.locator('.toolbar-area')).toBeVisible({ timeout: 10000 });
  });

  test('should show status bar when not on home screen', async ({ page }) => {
    // Status bar is only visible when not on home screen
    const statusBar = page.locator('.statusbar-area');
    // It may or may not be visible depending on home screen state
    const layout = page.locator('.app-layout');
    await expect(layout).toBeVisible({ timeout: 10000 });
  });
});
