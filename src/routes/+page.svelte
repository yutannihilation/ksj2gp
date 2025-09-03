<script lang="ts">
  import { onMount } from 'svelte';
  // Use Bits UI for a nicer error dialog
  // Note: ensure `bits-ui` is installed locally
  import { Dialog } from 'bits-ui';

  let inputEl: HTMLInputElement;
  let dragover = false;
  let busy = false;
  let worker: Worker | null = null;

  // Error dialog state
  let errorOpen = false;
  let errorMessage = '';

  onMount(() => {
    worker = new Worker(new URL('$lib/worker.ts', import.meta.url), { type: 'module' });

    worker.onmessage = async (event: MessageEvent) => {
      const data: any = event.data;

      const finish = () => {
        busy = false;
      };

      if (data && typeof data === 'object' && 'error' in data) {
        showError(String(data.error ?? '変換に失敗しました'));
        finish();
        return;
      }

      const fileHandle: any = data?.handle ?? data;
      if (!fileHandle || typeof fileHandle.getFile !== 'function') {
        showError('予期しない応答を受け取りました');
        finish();
        return;
      }

      const file = await fileHandle.getFile();
      const url = URL.createObjectURL(file);

      const a = document.createElement('a');
      a.href = url;
      a.download = file.name || 'tmp.parquet';
      document.body.appendChild(a);
      a.click();

      setTimeout(() => {
        URL.revokeObjectURL(url);
        a.remove();
        finish();
      }, 600);
    };

    return () => {
      worker?.terminate();
      worker = null;
    };
  });

  function pick() {
    inputEl?.click();
  }

  function processFile(file: File | undefined | null) {
    if (!file || !worker) return;
    busy = true;
    worker.postMessage({ file });
  }

  function onInputChange(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    processFile(file ?? null);
  }

  // Drag & drop handlers
  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragover = true;
  }

  function onDragLeave() {
    dragover = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragover = false;
    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    if (!file.name.toLowerCase().endsWith('.zip')) {
      showError('ZIP ファイルを選択してください');
      return;
    }
    processFile(file);
  }

  function showError(message: string) {
    errorMessage = message;
    errorOpen = true;
  }
</script>

<div class="page">
  <header class="header">
    <h1>KSJ → GeoParquet</h1>
    <p>国土数値情報の ZIP をドラッグ＆ドロップすると、GeoParquet に変換します。</p>
  </header>

  <section class="panel">
    <div
      id="dropzone"
      class="dropzone {dragover ? 'dragover' : ''}"
      role="button"
      tabindex="0"
      aria-label="ファイルのドラッグ＆ドロップ領域"
      on:dragover={onDragOver}
      on:dragleave={onDragLeave}
      on:drop={onDrop}
      on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && pick()}
    >
      <div class="dz-icon" aria-hidden="true">📦</div>
      <div class="dz-text"><strong>ここに ZIP をドロップ</strong><br />または下のボタンから選択</div>

      <div class="actions">
        <button type="button" class="btn" on:click|stopPropagation={pick} disabled={busy}>
          {busy ? '変換中…' : 'ZIP を選択'}
        </button>
        <input bind:this={inputEl} type="file" accept=".zip" hidden on:change={onInputChange} />
      </div>

      {#if busy}
        <div class="busy">
          <span class="spinner" aria-hidden="true"></span>
          <span class="sr-only">処理中</span>
        </div>
      {/if}
    </div>
  </section>

  <!-- Bits UI: Error dialog -->
  <Dialog.Root bind:open={errorOpen}>
    <Dialog.Content class="dialog">
      <Dialog.Title class="dialog-title">エラーが発生しました</Dialog.Title>
      <div class="dialog-body">{errorMessage}</div>
      <div class="dialog-actions">
        <Dialog.Close asChild>
          <button class="btn">閉じる</button>
        </Dialog.Close>
      </div>
    </Dialog.Content>
    <Dialog.Overlay class="overlay" />
  </Dialog.Root>
</div>

<style>
  :root {
    --bg: #0b1020;
    --panel: #121932;
    --panel-border: #233055;
    --text: #e7ecff;
    --muted: #a9b5d6;
    --accent: #6ea8ff;
    --accent-900: #2a5bd4;
    --success: #2dd4bf;
    --ring: rgba(110, 168, 255, 0.35);
  }

  .page {
    min-height: 100dvh;
    background: radial-gradient(1200px 400px at 20% -10%, #1b2550 0%, transparent 60%),
      radial-gradient(900px 360px at 80% -20%, #0f7ec9 0%, transparent 60%),
      var(--bg);
    color: var(--text);
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 24px;
    padding: 32px 20px 48px;
  }

  .header {
    text-align: center;
    max-width: 860px;
    margin: 0 auto;
  }
  .header h1 {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: 0.3px;
    margin: 0 0 6px;
  }
  .header p {
    margin: 0;
    color: var(--muted);
  }

  .panel {
    max-width: 860px;
    margin: 0 auto;
    background: linear-gradient(180deg, rgba(255,255,255,0.06), rgba(255,255,255,0.02));
    border: 1px solid var(--panel-border);
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.25), inset 0 1px 0 rgba(255,255,255,0.06);
    backdrop-filter: blur(6px);
  }

  .dropzone {
    position: relative;
    display: grid;
    place-items: center;
    gap: 14px;
    padding: 36px 18px;
    border: 2px dashed #38508a;
    border-radius: 14px;
    background-color: rgba(10, 18, 40, 0.6);
    outline: none;
    transition: border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
  }
  .dropzone.dragover {
    border-color: var(--accent);
    box-shadow: 0 0 0 6px var(--ring);
    transform: translateY(-1px);
  }
  .dz-icon { font-size: 42px; }
  .dz-text { color: var(--muted); text-align: center; line-height: 1.5; }

  .actions { display: flex; gap: 10px; }

  .btn {
    appearance: none;
    border: 0;
    border-radius: 10px;
    background: linear-gradient(180deg, var(--accent) 0%, var(--accent-900) 100%);
    color: white;
    padding: 10px 16px;
    font-weight: 600;
    letter-spacing: 0.3px;
    cursor: pointer;
    box-shadow: 0 6px 16px rgba(64, 149, 255, 0.35), inset 0 1px 0 rgba(255,255,255,0.35);
    transition: transform 0.1s ease, filter 0.1s ease, opacity 0.1s ease;
  }
  .btn:hover { filter: brightness(1.05); transform: translateY(-1px); }
  .btn:active { transform: translateY(0); filter: brightness(0.98); }
  .btn:disabled { opacity: 0.6; cursor: not-allowed; transform: none; }

  .busy {
    position: absolute;
    right: 12px;
    bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    font-size: 14px;
  }
  .spinner {
    width: 18px; height: 18px;
    border: 2px solid rgba(255,255,255,0.25);
    border-top-color: var(--success);
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .sr-only {
    position: absolute;
    width: 1px; height: 1px;
    padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0);
    white-space: nowrap; border: 0;
  }

  /* Bits UI dialog styling */
  .overlay {
    position: fixed; inset: 0; background: rgba(5,10,20,0.5);
    backdrop-filter: blur(2px);
  }
  .dialog {
    position: fixed; inset: 0; display: grid; place-items: center;
  }
  .dialog > * {
    background: #10172a; color: var(--text); border: 1px solid var(--panel-border);
    border-radius: 12px; padding: 18px; width: min(520px, calc(100% - 32px));
    box-shadow: 0 10px 30px rgba(0,0,0,0.4);
  }
  .dialog-title { font-weight: 700; margin-bottom: 6px; }
  .dialog-body { color: var(--muted); margin-bottom: 12px; }
  .dialog-actions { display: flex; justify-content: flex-end; }
</style>
