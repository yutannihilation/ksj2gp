import { convert_shp_to_geoparquet, IntermediateFiles } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data as { file: File };

  const opfsRoot = await navigator.storage.getDirectory();

  const outputFileHandle = await opfsRoot.getFileHandle("tmp.parquet", {
    create: true,
  });
  const outputFile = await outputFileHandle.createSyncAccessHandle();

  // TODO: use random file names
  const shp = await newSyncAccessHandle(opfsRoot, "tmp.shp");
  const dbf = await newSyncAccessHandle(opfsRoot, "tmp.dbf");
  const shx = await newSyncAccessHandle(opfsRoot, "tmp.shx");

  const intermediate_files = new IntermediateFiles(shp, dbf, shx);

  try {
    convert_shp_to_geoparquet(file, intermediate_files, outputFile);
    // Success: send handle in a stable envelope
    postMessage({ ok: true, handle: outputFileHandle });
  } catch (e: any) {
    const msg = typeof e === "string" ? e : e?.message ?? "unknown error";
    postMessage({ ok: false, error: msg });
  } finally {
    // Ensure handles are closed even on failure
    outputFile.close();
    shp.close();
    dbf.close();
    shx.close();
  }
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
