import { list_files, IntermediateFiles } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data;

  const opfsRoot = await navigator.storage.getDirectory();

  const outputFileHandle = await opfsRoot.getFileHandle("tmp.geoparquet", {
    create: true,
  });
  const outputFile = await outputFileHandle.createSyncAccessHandle();

  // TODO: use random file names
  const shp = await newSyncAccessHandle(opfsRoot, "tmp.shp");
  const dbf = await newSyncAccessHandle(opfsRoot, "tmp.dbf");
  const shx = await newSyncAccessHandle(opfsRoot, "tmp.shx");

  const intermediate_files = new IntermediateFiles(shp, dbf, shx);

  list_files(file, intermediate_files, outputFile);

  // TODO: can this be done automatically?
  outputFile.close();
  shp.close();
  dbf.close();
  shx.close();

  postMessage(outputFileHandle);
};

const newSyncAccessHandle = async (
  opfsRoot: FileSystemDirectoryHandle,
  filename: string
): Promise<FileSystemSyncAccessHandle> => {
  const fileHandle = await opfsRoot.getFileHandle(filename, {
    create: true,
  });

  return await fileHandle.createSyncAccessHandle();
};
