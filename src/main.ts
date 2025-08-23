const worker = new Worker(new URL("./worker.ts", import.meta.url), {
  type: "module", // It seems this is necessary
});

document.getElementById("file_picker")!.addEventListener(
  "change",
  function () {
    let file = this.files[0];
    worker.postMessage({ file });
  },
  false
);
