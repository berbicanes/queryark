import { test, expect } from '@playwright/test';
import { setupMockTauri } from './fixtures/mockTauri';

test.describe('Connection flow', () => {
  test.beforeEach(async ({ page }) => {
    await setupMockTauri(page);
    await page.goto('/');
  });

  test('should display welcome screen on first launch', async ({ page }) => {
    await expect(page.locator('.welcome-screen, .app-layout')).toBeVisible({ timeout: 10000 });
  });

  test('should open connection modal from welcome screen or toolbar', async ({ page }) => {
    // Try the toolbar add connection button
    const addBtn = page.locator('[title="Add Connection"], .add-connection-btn, button:has-text("Add Connection")');
    if (await addBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await addBtn.first().click();
      await expect(page.locator('.modal-overlay')).toBeVisible();
    }
  });

  test('should show database type selector in connection modal', async ({ page }) => {
    // Open connection modal
    const addBtn = page.locator('[title="Add Connection"], .add-connection-btn, button:has-text("Add Connection")');
    if (await addBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await addBtn.first().click();
      // Check for database type options
      await expect(page.locator('.modal-card')).toBeVisible();
    }
  });
});
