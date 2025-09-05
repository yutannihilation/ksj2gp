import {
	convert_shp_to_geoparquet,
	convert_shp_to_geojson,
	IntermediateFiles,
	list_shp_files
} from 'ksj2gp';
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

function getOutputFilename(x: string, ext: string): string {
	const start = x.lastIndexOf('/') + 1;
	const end = x.lastIndexOf('.') + 1;
	return x.substring(start, end) + ext;
}

console.log('Worker loaded');

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const file = event.data.file;
	const output_format = event.data.output_format;
	let target_shp = event.data.target_shp;

	if (!target_shp) {
		const shp_file_candidates = list_shp_files(file);

		if (shp_file_candidates.length == 0) {
			postTypedMessage({ error: 'No .shp files found in the archive' });
			return;
		} else if (shp_file_candidates.length == 1) {
			target_shp = shp_file_candidates[0];
		} else {
			// Return available .shp files to the main thread so UI can prompt user
			postTypedMessage({ shp_file_candidates });
			return; // Wait for a follow-up message with target_shp
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
		let file_ext = '';
		if (output_format === 'GeoParquet') {
			convert_shp_to_geoparquet(file, target_shp, intermediate_files, outputFile);
			file_ext = 'parquet';
		} else if (output_format === 'GeoJson') {
			convert_shp_to_geojson(file, target_shp, intermediate_files, outputFile);
			file_ext = 'geojson';
		}
		// Success: send handle in a stable envelope
		const filename = getOutputFilename(target_shp, file_ext);
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
