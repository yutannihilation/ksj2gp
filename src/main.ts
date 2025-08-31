const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });

const input = document.getElementById("file_picker") as HTMLInputElement;
const dropzone = document.getElementById("dropzone") as HTMLDivElement;
const browseBtn = document.getElementById("browse_btn") as HTMLButtonElement;
const processFile = (file: File) => {
  if (!file) return;
  browseBtn.disabled = true;
  input.disabled = true;
  worker.postMessage({ file });
};

// Browse button and clicking the dropzone open the file picker
browseBtn.addEventListener("click", () => input.click());
dropzone.addEventListener("click", () => input.click());

// Input change -> process file
input.addEventListener("change", function () {
  const file = this.files?.[0];
  if (file) processFile(file);
});

// Drag & drop behaviors
["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
  document.addEventListener(eventName, (e) => {
    e.preventDefault();
    e.stopPropagation();
  });
});

dropzone.addEventListener("dragover", () => dropzone.classList.add("dragover"));
dropzone.addEventListener("dragenter", () => dropzone.classList.add("dragover"));
dropzone.addEventListener("dragleave", () => dropzone.classList.remove("dragover"));
dropzone.addEventListener("drop", (e: DragEvent) => {
  dropzone.classList.remove("dragover");
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) processFile(files[0]);
});

// Handle results from worker (both success and error)
worker.onmessage = async (event) => {
  const data: any = event.data;

  const finishUI = () => {
    browseBtn.disabled = false;
    input.disabled = false;
  };

  if (data && typeof data === "object" && "error" in data) {
    alert(`エラー: ${data.error}`);
    finishUI();
    return;
  }

  // Backward-compat: worker may send the handle directly, or wrapped in { ok, handle }
  const fileHandle: any = data?.handle ?? data;
  if (!fileHandle || typeof fileHandle.getFile !== "function") {
    alert("予期しない応答を受け取りました");
    finishUI();
    return;
  }

  const file = await fileHandle.getFile();
  const url = URL.createObjectURL(file);

  // Trigger download
  const a = document.createElement("a");
  a.href = url;
  a.download = file.name || "tmp.parquet";
  document.body.appendChild(a);
  a.click();

  setTimeout(() => {
    URL.revokeObjectURL(url);
    a.remove();
    finishUI();
  }, 600);
};
