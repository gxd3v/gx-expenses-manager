import { invoke } from '@tauri-apps/api/core';

type GraphQLResponse<T> = {
	data?: T | null;
	errors?: { message: string }[];
};

export async function gql<T>(query: string, variables: Record<string, unknown> = {}): Promise<T> {
	const response = await invoke<GraphQLResponse<T>>('graphql', { request: { query, variables } });
	if (response.errors?.length) {
		throw new Error(response.errors.map((e) => e.message).join('\n'));
	}
	return response.data as T;
}
