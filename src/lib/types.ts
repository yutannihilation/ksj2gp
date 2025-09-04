export type WorkerRequest = {
	// ZIP file
	file: File;
	// path to Shapefile in the ZIP file
	target_shp?: string;
};

export type WorkerResponse = {
    ready?: boolean;
    error?: string;
    handle?: FileSystemFileHandle;
    // When multiple .shp files are detected in the archive, the worker returns
    // the list so the main thread can prompt the user to choose one.
    shp_files?: string[];
};
