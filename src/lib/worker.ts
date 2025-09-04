import { convert_shp_to_geoparquet, IntermediateFiles, list_shp_files } from 'ksj2gp';
import type { WorkerRequest, WorkerResponse } from './types';

function postTypedMessage(message: WorkerResponse) {
	postMessage(message);
}

// Notify main thread that the worker bundle is ready to accept messages
// This allows the UI to enable inputs only after initialization
postTypedMessage({ ready: true });

async function newSyncAccessHandle(
	opfsRoot: FileSystemDirectoryHandle,
	filename: string
): Promise<FileSystemSyncAccessHandle> {
	const fileHandle = await opfsRoot.getFileHandle(filename, {
		create: true
	});

	return await fileHandle.createSyncAccessHandle();
}

console.log('Worker loaded');

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const file = event.data.file;
	let target_shp = event.data.target_shp;

	if (!target_shp) {
		const shp_files = list_shp_files(file);

		if (shp_files.length == 0) {
			postTypedMessage({ error: 'No .shp files found in the archive' });
			return;
		} else if (shp_files.length == 1) {
			target_shp = shp_files[0];
		} else {
			// Return available .shp files to the main thread so UI can prompt user
			postTypedMessage({ shp_files });
			return; // Wait for a follow-up message with target_shp
		}
	}

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
		convert_shp_to_geoparquet(file, target_shp, intermediate_files, outputFile);
		// Success: send handle in a stable envelope
		postTypedMessage({ handle: outputFileHandle });
	} catch (e: unknown) {
		const msg =
			typeof e === 'string'
				? e
				: typeof (e as { message?: unknown })?.message === 'string'
					? (e as { message: string }).message
					: 'unknown error';
		postTypedMessage({ error: msg });
	} finally {
		// Ensure handles are closed even on failure
		outputFile.close();
		shp.close();
		dbf.close();
		shx.close();
	}
};
