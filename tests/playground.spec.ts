import { expect, test } from '@playwright/test';

test('実際のWasmがサブパスで読み込まれ、結果を繰り返し表示する', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  const wasmResponse = page.waitForResponse(response => response.url().endsWith('.wasm'));
  await page.goto('./');
  expect((await wasmResponse).ok()).toBe(true);
  await expect(page.getByRole('button', { name: '実行する' })).toBeEnabled();
  await page.getByLabel('MLの式').fill('42');
  await page.getByRole('button', { name: '実行する' }).click();
  await expect(page.getByRole('status')).toContainText('処理が完了');
  // 実装が進んでも使える基盤テスト。初期状態では「未実装」、実装後は「成功」。
  await expect(page.locator('[data-stage="tokens"] .badge')).toHaveText(/^(未実装|成功)$/);
  await expect(page.locator('[data-stage="tokens"] pre')).not.toBeEmpty();
  await expect(page.locator('.badge[data-status="error"]')).toHaveCount(0);
  await page.getByLabel('サンプル', { exact: true }).selectOption('2');
  await expect(page.locator('.badge[data-status="skipped"]')).toHaveCount(4);
  await page.getByRole('button', { name: '実行する' }).click();
  await expect(page.getByRole('status')).toContainText('処理が完了');
  expect(errors).toEqual([]);
});

test('応答のないWorkerを中止し、新しいWasmで再実行できる', async ({ page }) => {
  await page.addInitScript(() => {
    // 最初のWorkerだけ要求を保留し、処理が返らない状況を再現する。
    // Wasm初期化と、中止後に作られるWorkerは実物を使う。
    const NativeWorker = window.Worker;
    let count = 0;
    window.Worker = class extends NativeWorker {
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        if (count++ === 0) this.postMessage = () => {};
      }
    };
  });
  await page.goto('./');
  await page.getByRole('button', { name: '実行する' }).click();
  await expect(page.getByRole('status')).toContainText('実行中');
  await expect(page.getByRole('button', { name: '実行する' })).toBeDisabled();
  await page.getByRole('button', { name: '中止', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('もう一度実行できます');
  await page.getByRole('button', { name: '実行する' }).click();
  await expect(page.getByRole('status')).toContainText('処理が完了');
});

test('Wasm読み込みの失敗を表示し、再接続できる', async ({ page }) => {
  let fail = true;
  await page.route('**/*.wasm', async route => {
    if (fail) {
      fail = false;
      await route.abort();
    } else {
      await route.continue();
    }
  });
  await page.goto('./');
  await expect(page.getByRole('status')).toContainText('実行環境エラー');
  await page.getByRole('button', { name: '再接続' }).click();
  await expect(page.getByRole('button', { name: '実行する' })).toBeEnabled();
  await page.getByRole('button', { name: '実行する' }).click();
  await expect(page.getByRole('status')).toContainText('処理が完了');
});

test('狭い画面で横にはみ出さず、キーボードから実行できる', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('./');
  await expect(page.getByRole('button', { name: '実行する' })).toBeEnabled();
  await page.getByLabel('MLの式').fill('42');
  await page.getByLabel('MLの式').press('Control+Enter');
  await expect(page.getByRole('status')).toContainText('処理が完了');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});
