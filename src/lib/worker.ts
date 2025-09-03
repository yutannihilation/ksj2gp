import { convert_shp_to_geoparquet, IntermediateFiles } from 'ksj2gp';
import type { WorkerRequest, WorkerResponse } from './types';

function postMessage(message: WorkerResponse) {
	window.postMessage(message);
}

// Notify main thread that the worker bundle is ready to accept messages
// This allows the UI to enable inputs only after initialization
postMessage({ ready: true });

async function newSyncAccessHandle(
	opfsRoot: FileSystemDirectoryHandle,
	filename: string
): Promise<FileSystemSyncAccessHandle> {
	const fileHandle = await opfsRoot.getFileHandle(filename, {
		create: true
	});

	return await fileHandle.createSyncAccessHandle();
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const { file } = event.data;

	const opfsRoot = await navigator.storage.getDirectory();

	const outputFileHandle = await opfsRoot.getFileHandle('tmp.parquet', {
		create: true
	});
	const outputFile = await outputFileHandle.createSyncAccessHandle();

	// TODO: use random file names
	const shp = await newSyncAccessHandle(opfsRoot, 'tmp.shp');
	const dbf = await newSyncAccessHandle(opfsRoot, 'tmp.dbf');
	const shx = await newSyncAccessHandle(opfsRoot, 'tmp.shx');

	const intermediate_files = new IntermediateFiles(shp, dbf, shx);

	try {
		convert_shp_to_geoparquet(file, intermediate_files, outputFile);
		// Success: send handle in a stable envelope
		postMessage({ handle: outputFileHandle });
	} catch (e: unknown) {
		const msg =
			typeof e === 'string'
				? e
				: typeof (e as { message?: unknown })?.message === 'string'
					? (e as { message: string }).message
					: 'unknown error';
		postMessage({ error: msg });
	} finally {
		// Ensure handles are closed even on failure
		outputFile.close();
		shp.close();
		dbf.close();
		shx.close();
	}
};
