import { convert_shp, IntermediateFiles, list_shp_files } from 'ksj2gp';
import type { OutputFormat, WorkerRequest, WorkerResponse } from './types';

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

function getOutputFilename(x: string, outputFormat: OutputFormat): string {
	let ext = '';
	if (outputFormat === 'GeoParquet') {
		ext = 'parquet';
	} else if (outputFormat === 'GeoJson') {
		ext = 'geojson';
	}

	const start = x.lastIndexOf('/') + 1;
	const end = x.lastIndexOf('.') + 1;
	return x.substring(start, end) + ext;
}

console.log('Worker loaded');

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const file = event.data.file;
	const outputFormat = event.data.outputFormat;
	let targetShp = event.data.targetShp;

	if (!targetShp) {
		const shpFileCandidates = list_shp_files(file);

		if (shpFileCandidates.length == 0) {
			postTypedMessage({ error: 'No .shp files found in the archive' });
			return;
		} else if (shpFileCandidates.length == 1) {
			targetShp = shpFileCandidates[0];
		} else {
			// Return available .shp files to the main thread so UI can prompt user
			postTypedMessage({ shpFileCandidates });
			return; // Wait for a follow-up message with targetShp
		}
	}

	const opfsRoot = await navigator.storage.getDirectory();

	// This file doesn't need to have a proper extension because the filename is
	// given on frontend anyway.
	const outputFileHandle = await opfsRoot.getFileHandle('tmp_output', {
		create: true
	});
	const outputFile = await outputFileHandle.createSyncAccessHandle();

	// TODO: use random file names
	const shp = await newSyncAccessHandle(opfsRoot, 'tmp.shp');
	const dbf = await newSyncAccessHandle(opfsRoot, 'tmp.dbf');
	const shx = await newSyncAccessHandle(opfsRoot, 'tmp.shx');

	const intermediate_files = new IntermediateFiles(shp, dbf, shx);

	try {
		convert_shp(file, targetShp, intermediate_files, outputFile, outputFormat);
		const filename = getOutputFilename(targetShp, outputFormat);
		postTypedMessage({ output: { handle: outputFileHandle, filename } });
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
