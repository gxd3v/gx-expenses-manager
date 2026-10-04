import { gql } from '../graphql.ts';
import { cleanFilter, type TransactionFilter } from './transactions.ts';

export type BackupFile = { path: string; name: string; createdAt: string; size: number };

export type ImportPreview = {
	formatVersion: number;
	schemaVersion: number;
	exportedAt: string | null;
	encrypted: boolean;
	counts: { table: string; rows: number }[];
	conflicts: number;
};

export async function listBackups(): Promise<BackupFile[]> {
	const data = await gql<{ backups: BackupFile[] }>(`{ backups { path name createdAt size } }`);
	return data.backups;
}

export async function createBackup(): Promise<BackupFile> {
	const data = await gql<{ createBackup: BackupFile }>(`mutation { createBackup { path name createdAt size } }`);
	return data.createBackup;
}

export async function autoBackup(): Promise<void> {
	await gql(`mutation { autoBackup { path } }`);
}

export async function verifyBackup(path: string, password: string | null): Promise<void> {
	await gql(`mutation ($path: String!, $password: String) { verifyBackup(path: $path, password: $password) }`, {
		path,
		password
	});
}

export async function exportData(path: string, password: string | null): Promise<void> {
	await gql(`mutation ($path: String!, $password: String) { exportData(path: $path, password: $password) }`, {
		path,
		password
	});
}

export async function exportCsv(path: string, filter: TransactionFilter): Promise<number> {
	const data = await gql<{ exportCsv: number }>(
		`mutation ($path: String!, $filter: TransactionFilter!) { exportCsv(path: $path, filter: $filter) }`,
		{ path, filter: cleanFilter(filter) }
	);
	return data.exportCsv;
}

export async function inspectImport(path: string, password: string | null): Promise<ImportPreview> {
	const data = await gql<{ inspectImport: ImportPreview }>(
		`query ($path: String!, $password: String) {
			inspectImport(path: $path, password: $password) { formatVersion schemaVersion exportedAt encrypted counts { table rows } conflicts }
		}`,
		{ path, password }
	);
	return data.inspectImport;
}

export async function importData(
	path: string,
	password: string | null,
	replace: boolean
): Promise<{ inserted: number; skipped: number }> {
	const data = await gql<{ importData: { inserted: number; skipped: number } }>(
		`mutation ($path: String!, $password: String, $replace: Boolean!) {
			importData(path: $path, password: $password, replace: $replace) { inserted skipped }
		}`,
		{ path, password, replace }
	);
	return data.importData;
}
