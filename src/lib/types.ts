export type WorkerRequest = { file: File };

export type WorkerResponse = {
	ready?: boolean;
	error?: string;
	handle?: FileSystemFileHandle;
};
