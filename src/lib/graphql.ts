import { invoke } from '@tauri-apps/api/core';

type GraphQLResponse<T> = {
	data?: T | null;
	errors?: { message: string }[];
};

type Request = {
	query: string;
	variables: Record<string, unknown>;
};

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function execute<T>(request: Request): Promise<GraphQLResponse<T>> {
	if (isTauri) return invoke<GraphQLResponse<T>>('graphql', { request });

	const response = await fetch('/graphql', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(request)
	});
	return response.json();
}

export async function gql<T>(query: string, variables: Record<string, unknown> = {}): Promise<T> {
	const response = await execute<T>({ query, variables });
	if (response.errors?.length) {
		throw new Error(response.errors.map((e) => e.message).join('\n'));
	}
	return response.data as T;
}

export function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}
