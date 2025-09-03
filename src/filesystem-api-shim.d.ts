// According to MDN, this API is available on all the major browsers since 2023.
// However, this is not available on TypeScript (I don't know the details). So,
// this type definition is neccessary to aboid compile error.
//
// cf. https://developer.mozilla.org/en-US/docs/Web/API/FileSystemSyncAccessHandle

interface FileSystemSyncAccessHandle {
	read(buffer: ArrayBuffer | ArrayBufferView, options?: { at?: number }): number;
	write(buffer: ArrayBuffer | ArrayBufferView, options?: { at?: number }): number;
	flush(): void;
	close(): void;
	getSize(): number;
}

interface FileSystemFileHandle {
	createSyncAccessHandle(): Promise<FileSystemSyncAccessHandle>;
}
