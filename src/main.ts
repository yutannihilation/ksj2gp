const worker = new Worker(new URL("./worker.ts", import.meta.url), {
  type: "module", // It seems this is necessary
});

const input = document.getElementById("file_picker") as HTMLInputElement;
input.addEventListener(
  "change",
  function () {
    if (this.files) {
      const file = this.files[0];
      worker.postMessage({ file });
    }
  },
  false
);

worker.onmessage = async (event) => {
  const fileHandle = event.data;
  const file = await fileHandle.getFile();
  const url = URL.createObjectURL(file);

  // Create a download link and trigger download
  const a = document.createElement("a");
  a.href = url;
  a.download = file.name || "download.txt";
  document.body.appendChild(a);
  a.click();

  // Clean up
  setTimeout(() => {
    URL.revokeObjectURL(url);
    a.remove();
  }, 1000);
};
