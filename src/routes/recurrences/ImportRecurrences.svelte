<script lang="ts">
	import { saveCategory } from '#lib/api/categories.ts';
	import { frequencyLabel, saveRecurrence } from '#lib/api/recurrences.ts';
	import Amount from '#lib/components/Amount.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import { memberRecurrences, type MemberRecurrence } from '#lib/session.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';
	import { current, membersOf } from '#lib/user.svelte.ts';

	let { onclose }: { onclose: () => void } = $props();

	type Row = { source: MemberRecurrence; selected: boolean; invert: boolean; accountId: string };

	const members = membersOf(current.user);
	const accounts = refs.accounts.filter((a) => !a.archivedAt);

	let memberId = $state(current.returnTo && members.some((m) => m.id === current.returnTo?.id) ? current.returnTo.id : (members[0]?.id ?? ''));
	let password = $state('');
	let rows = $state<Row[] | null>(null);
	let busy = $state(false);
	let error = $state('');

	const remembered = $derived(current.returnTo?.id === memberId);
	const chosen = $derived(rows?.filter((r) => r.selected) ?? []);

	async function load(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = '';
		try {
			const list = await memberRecurrences(memberId, remembered ? null : password);
			rows = list.map((source) => ({ source, selected: false, invert: false, accountId: accounts[0]?.id ?? '' }));
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}

	const flip = (kind: MemberRecurrence['kind']) => (kind === 'INCOME' ? 'OUTCOME' : 'INCOME');

	async function category(path: string | null, kind: MemberRecurrence['kind'], created: Map<string, string>): Promise<string | null> {
		if (!path) return null;
		let parentId: string | null = null;
		for (const name of path.split(' / ')) {
			const key: string = `${parentId}/${name.toLowerCase()}`;
			const existing = refs.categories.find(
				(c) => c.parentId === parentId && c.name.toLowerCase() === name.toLowerCase() && c.kind !== flip(kind)
			);
			const id: string = created.get(key) ?? existing?.id ?? (await saveCategory(null, { parentId, name, kind, icon: null, color: null })).id;
			created.set(key, id);
			parentId = id;
		}
		return parentId;
	}

	async function importChosen() {
		busy = true;
		const created = new Map<string, string>();
		try {
			for (const { source, invert, accountId } of chosen) {
				const kind = invert ? flip(source.kind) : source.kind;
				await saveRecurrence(null, {
					accountId,
					categoryId: invert ? null : await category(source.categoryName, kind, created),
					kind,
					amount: source.amount,
					description: source.description,
					startDate: source.startDate,
					endDate: source.endDate,
					unit: source.unit,
					interval: source.interval,
					toAccountId: null,
					variableAmount: source.variableAmount
				});
			}
			notify(`${chosen.length} ${chosen.length === 1 ? 'recorrência importada' : 'recorrências importadas'}`);
			await dataChanged();
			onclose();
		} catch (e) {
			notifyError(e);
			await dataChanged();
		} finally {
			busy = false;
		}
	}
</script>

<Modal title="Importar recorrências de um membro" {onclose} wide>
	{#if rows === null}
		<form onsubmit={load} class="space-y-4">
			<label class="label">
				Membro
				<select bind:value={memberId} class="input">
					{#each members as member (member.id)}<option value={member.id}>{member.name}</option>{/each}
				</select>
			</label>
			{#if !remembered}
				<label class="label">
					Password do membro
					<input type="password" bind:value={password} class="input" autocomplete="off" required />
				</label>
			{/if}
			{#if error}<p class="text-sm text-red-600" role="alert">{error}</p>{/if}
			<div class="flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={onclose}>Cancelar</button>
				<button type="submit" class="btn-primary" disabled={busy || !memberId}>{busy ? 'A abrir…' : 'Ver recorrências'}</button>
			</div>
		</form>
	{:else if rows.length === 0}
		<p class="muted">Este membro não tem recorrências ativas.</p>
	{:else}
		<div class="space-y-4">
			<p class="muted">
				As recorrências escolhidas são copiadas para esta conta; nada muda no membro. "Inverter" transforma uma saída do membro numa entrada
				aqui (por exemplo, uma ajuda à casa passa a contribuição) e deixa-a sem categoria. Créditos e transferências passam só com o nome, o
				valor e a periodicidade.
			</p>
			<div class="max-h-[50vh] overflow-auto">
				<table class="table-base">
					<thead>
						<tr>
							<th class="w-8"></th>
							<th>Descrição</th>
							<th>Periodicidade</th>
							<th class="text-right">Valor</th>
							<th class="text-center">Inverter</th>
							<th>Conta de destino</th>
						</tr>
					</thead>
					<tbody>
						{#each rows as row, index (index)}
							<tr>
								<td><input type="checkbox" bind:checked={row.selected} aria-label="Importar {row.source.description}" /></td>
								<td>
									{row.source.description}
									{#if row.source.linked}<span class="badge ml-1">sem ligação</span>{/if}
									<p class="text-xs text-stone-500">{row.source.categoryName ?? 'Sem categoria'} · {row.source.accountName}</p>
								</td>
								<td>{frequencyLabel(row.source.unit, row.source.interval)}</td>
								<td class="text-right">
									<Amount value={(row.invert ? flip(row.source.kind) : row.source.kind) === 'INCOME' ? row.source.amount : -row.source.amount} />
								</td>
								<td class="text-center"><input type="checkbox" bind:checked={row.invert} aria-label="Inverter sentido" /></td>
								<td>
									<select bind:value={row.accountId} class="input py-1" disabled={!row.selected}>
										{#each accounts as account (account.id)}<option value={account.id}>{account.name}</option>{/each}
									</select>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			{#if accounts.length === 0}
				<p class="text-sm text-red-600">É necessário criar primeiro uma conta nesta conta conjunta.</p>
			{/if}
			<div class="flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={onclose}>Cancelar</button>
				<button class="btn-primary" disabled={busy || chosen.length === 0 || accounts.length === 0} onclick={importChosen}>
					{busy ? 'A importar…' : `Importar ${chosen.length}`}
				</button>
			</div>
		</div>
	{/if}
</Modal>
