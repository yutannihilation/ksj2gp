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
};
