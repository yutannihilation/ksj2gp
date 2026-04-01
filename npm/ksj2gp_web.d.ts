/* tslint:disable */
/* eslint-disable */

export class IntermediateFiles {
	free(): void;
	[Symbol.dispose](): void;
	constructor(
		shp: FileSystemSyncAccessHandle,
		dbf: FileSystemSyncAccessHandle,
		shx: FileSystemSyncAccessHandle
	);
}

export class Point {
	free(): void;
	[Symbol.dispose](): void;
	constructor(x: number, y: number, z: number);
	x: number;
	y: number;
	z: number;
}

export class Projection {
	free(): void;
	[Symbol.dispose](): void;
	constructor(defn: string);
	readonly axis: string;
	readonly isGeocentric: boolean;
	readonly isLatlon: boolean;
	readonly isNormalizedAxis: boolean;
	readonly projName: string;
	readonly to_meter: number;
	readonly units: string;
}

/**
 * Read a binary NTv2 from Dataview.
 *
 * Note: only NTv2 file format are supported.
 */
export function add_nadgrid(key: string, view: DataView): void;

export function convert_shp(
	zip_file: File,
	target_shp: string,
	intermediate_files: IntermediateFiles,
	output_file: FileSystemSyncAccessHandle,
	output_format: string,
	translate_colnames: boolean,
	translate_contents: boolean,
	ignore_translation_errors: boolean
): void;

export function list_shp_files(zip_file: File): string[];

export function transform(src: Projection, dst: Projection, point: Point): void;
