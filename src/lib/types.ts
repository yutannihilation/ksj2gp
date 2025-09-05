export type WorkerRequest = {
	// ZIP file
	file: File;
	// format of output file
	outputFormat: OutputFormat;
	// path to Shapefile in the ZIP file
	target_shp?: string;
};

export type OutputFormat = 'GeoParquet' | 'GeoJson';

export type ResultFile = {
	handle: FileSystemFileHandle;
	filename: string;
};

export type WorkerResponse = {
	ready?: boolean;
	error?: string;
	// When multiple .shp files are detected in the archive, the worker returns
	// the list so the main thread can prompt the user to choose one.
	shp_file_candidates?: string[];
	// Result file (e.g. GeoParquet)
	output: ResultFile;
};
