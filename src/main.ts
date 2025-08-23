const worker = new Worker(new URL("./worker.ts", import.meta.url), {
  type: "module", // It seems this is necessary
});

const input = document.getElementById("file_picker") as HTMLInputElement;
input.addEventListener(
  "change",
  function () {
    if (this.files) {
      let file = this.files[0];
      worker.postMessage({ file });
    }
  },
  false
);
