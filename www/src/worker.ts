import { list_files } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data;

  const opfsRoot = await navigator.storage.getDirectory();
  const fileHandle = await opfsRoot.getFileHandle("tmp.shp", { create: true });

  const tmp_shp_file = await fileHandle.createSyncAccessHandle();

  list_files(file, tmp_shp_file);
};
