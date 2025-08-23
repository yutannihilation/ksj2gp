import { list_files, IntermediateFiles } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data;

  const opfsRoot = await navigator.storage.getDirectory();

  // TODO: use random file names
  const shp = await newSyncAccessHandle(opfsRoot, "tmp.shp");
  const dbf = await newSyncAccessHandle(opfsRoot, "tmp.dbf");
  const shx = await newSyncAccessHandle(opfsRoot, "tmp.shx");

  const intermediate_files = new IntermediateFiles(shp, dbf, shx);

  list_files(file, intermediate_files);

  // TODO: can this be done automatically?
  shp.close();
  dbf.close();
  shx.close();
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
