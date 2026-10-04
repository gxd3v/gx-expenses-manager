<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import {
		createBackup,
		exportData,
		importData,
		inspectImport,
		listBackups,
		verifyBackup,
		type BackupFile,
		type ImportPreview
	} from '#lib/api/backups.ts';
	import Modal from '#lib/components/Modal.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import { confirmAction, pickDirectory, pickOpenPath, pickSavePath } from '#lib/dialogs.ts';
	import { formatDate, today } from '#lib/format.ts';
	import { errorMessage, isTauri } from '#lib/graphql.ts';
	import { dataChanged } from '#lib/refs.svelte.ts';
	import { changePassword, restoreBackup, status } from '#lib/session.ts';
	import { app, saveSettings, type Settings } from '#lib/settings.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let form = $state<Settings>(untrack(() => ({ ...app.settings! })));
	let backups = $state<BackupFile[]>([]);
	let dataDir = $state('');
	let passwords = $state({ current: '', next: '', confirm: '' });
	let exportPassword = $state('');
	let busy = $state(false);
	let importing = $state<{ path: string; password: string; preview: ImportPreview | null } | null>(null);

	async function loadBackups() {
		backups = await listBackups().catch(() => []);
	}

	onMount(async () => {
		await loadBackups();
		dataDir = (await status()).dataDir;
	});

	async function save(event: SubmitEvent) {
		event.preventDefault();
		try {
			form = { ...(await saveSettings(form)) };
			notify('Definições guardadas');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function chooseBackupDir() {
		const dir = await pickDirectory();
		if (dir) form.backupDir = dir;
	}

	async function backupNow() {
		try {
			const backup = await createBackup();
			notify(`Backup criado: ${backup.name}`);
			await loadBackups();
		} catch (e) {
			notifyError(e);
		}
	}

	async function verify(backup: BackupFile) {
		try {
			await verifyBackup(backup.path, null);
			notify('Backup íntegro');
		} catch (e) {
			notify(`Backup com problemas: ${errorMessage(e)}`, 'error');
		}
	}

	async function restore(backup: BackupFile) {
		const message = `Restaurar "${backup.name}"? Os dados atuais são substituídos (é guardada uma cópia de segurança antes).`;
		if (!(await confirmAction(message))) return;
		const password = window.prompt('Password do backup (deixa vazio se for a atual)') || null;
		try {
			await restoreBackup(backup.path, password);
			notify('Backup restaurado');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function exportAll() {
		const path = await pickSavePath(`gx-expenses-${today()}.gxbackup`, 'gxbackup');
		if (!path) return;
		busy = true;
		try {
			await exportData(path, exportPassword || null);
			notify(exportPassword ? 'Exportação encriptada criada' : 'Exportação criada (não encriptada)');
			exportPassword = '';
		} catch (e) {
			notifyError(e);
		} finally {
			busy = false;
		}
	}

	async function chooseImport() {
		const path = await pickOpenPath(['gxbackup', 'json']);
		if (path) importing = { path, password: '', preview: null };
	}

	async function preview() {
		if (!importing) return;
		busy = true;
		try {
			importing.preview = await inspectImport(importing.path, importing.password || null);
		} catch (e) {
			notifyError(e);
		} finally {
			busy = false;
		}
	}

	async function runImport(replace: boolean) {
		if (!importing) return;
		const message = replace ? 'Substituir TODOS os dados atuais pelos do ficheiro?' : 'Juntar os dados do ficheiro aos atuais?';
		if (!(await confirmAction(message))) return;
		busy = true;
		try {
			const result = await importData(importing.path, importing.password || null, replace);
			notify(`Importação concluída: ${result.inserted} registos novos, ${result.skipped} ignorados`);
			importing = null;
			await dataChanged();
		} catch (e) {
			notifyError(e);
		} finally {
			busy = false;
		}
	}

	async function updatePassword(event: SubmitEvent) {
		event.preventDefault();
		if (passwords.next !== passwords.confirm) {
			notify('As novas passwords não coincidem', 'error');
			return;
		}
		try {
			await changePassword(passwords.current, passwords.next);
			passwords = { current: '', next: '', confirm: '' };
			notify('Password alterada. Backups antigos continuam a usar a password antiga.');
		} catch (e) {
			notifyError(e);
		}
	}

	const sizeLabel = (bytes: number) => `${(bytes / 1024).toFixed(0)} KB`;
</script>

<PageHeader title="Definições" />

<form onsubmit={save} class="space-y-6">
	<section class="card grid grid-cols-2 gap-4 md:grid-cols-4">
		<h2 class="col-span-2 font-medium md:col-span-4">Geral</h2>
		<label class="label">
			Moeda principal
			<input bind:value={form.currency} maxlength="3" class="input uppercase" />
		</label>
		<label class="label">
			Formato de data
			<select bind:value={form.dateFormat} class="input">
				<option value="DAY_MONTH_YEAR">dd/mm/aaaa</option>
				<option value="YEAR_MONTH_DAY">aaaa-mm-dd</option>
				<option value="MONTH_DAY_YEAR">mm/dd/aaaa</option>
			</select>
		</label>
		<label class="label">
			Primeiro dia da semana
			<select bind:value={form.firstDayOfWeek} class="input">
				<option value={1}>Segunda-feira</option>
				<option value={0}>Domingo</option>
				<option value={6}>Sábado</option>
			</select>
		</label>
		<label class="label">
			Tema
			<select bind:value={form.theme} class="input">
				<option value="SYSTEM">Sistema</option>
				<option value="LIGHT">Claro</option>
				<option value="DARK">Escuro</option>
			</select>
		</label>
	</section>

	<section class="card grid grid-cols-2 gap-4 md:grid-cols-4">
		<h2 class="col-span-2 font-medium md:col-span-4">Previsões</h2>
		<label class="label">
			Método por omissão
			<select bind:value={form.forecastMethod} class="input">
				<option value="HISTORY">Recorrências + histórico</option>
				<option value="RECURRING">Só recorrências</option>
			</select>
		</label>
		<label class="label">
			Meses de histórico
			<input type="number" min="1" max="24" bind:value={form.forecastHistoryMonths} class="input" />
		</label>
		<label class="label">
			Horizonte no dashboard (meses)
			<input type="number" min="1" max="120" bind:value={form.forecastHorizonMonths} class="input" />
		</label>
	</section>

	<section class="card grid grid-cols-2 gap-4 md:grid-cols-4">
		<h2 class="col-span-2 font-medium md:col-span-4">Notificações</h2>
		<label class="col-span-2 flex items-center gap-2 text-sm md:col-span-4">
			<input type="checkbox" bind:checked={form.notificationsEnabled} class="rounded" /> Ativar avisos e notificações
		</label>
		{#each [['notifyUpcoming', 'Despesas e rendimentos recorrentes próximos'], ['notifyCredits', 'Prestações de créditos'], ['notifyGoals', 'Objetivos'], ['notifyLowBalance', 'Saldo baixo'], ['notifyNegativeForecast', 'Previsões negativas']] as const as [key, label] (key)}
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={form[key]} disabled={!form.notificationsEnabled} class="rounded" />
				{label}
			</label>
		{/each}
		<label class="label">
			Avisar com antecedência (dias)
			<input type="number" min="1" max="60" bind:value={form.notifyDaysAhead} class="input" />
		</label>
		<label class="label">
			Limite de saldo baixo
			<MoneyInput bind:value={form.lowBalanceThreshold as number | null} />
		</label>
	</section>

	<section class="card grid grid-cols-2 gap-4 md:grid-cols-4">
		<h2 class="col-span-2 font-medium md:col-span-4">Segurança e backups automáticos</h2>
		<label class="label">
			Bloqueio automático (minutos, 0 = nunca)
			<input type="number" min="0" max="1440" bind:value={form.lockTimeoutMinutes} class="input" />
		</label>
		<label class="label">
			Backup automático (dias, 0 = desligado)
			<input type="number" min="0" max="365" bind:value={form.backupFrequencyDays} class="input" />
		</label>
		<label class="label">
			Backups automáticos a manter
			<input type="number" min="1" max="100" bind:value={form.backupKeep} class="input" />
		</label>
		<div class="label">
			Pasta dos backups
			<div class="flex gap-2">
				<input value={form.backupDir ?? 'Pasta de dados da aplicação'} readonly class="input" />
				<button type="button" class="btn-secondary" onclick={chooseBackupDir}>Escolher</button>
				{#if form.backupDir}<button type="button" class="btn-ghost" onclick={() => (form.backupDir = null)}>Repor</button>{/if}
			</div>
		</div>
	</section>

	<div class="flex justify-end"><button class="btn-primary">Guardar definições</button></div>
</form>

<section class="card mt-6">
	<header class="mb-3 flex items-center justify-between">
		<div>
			<h2 class="font-medium">Backups</h2>
			<p class="muted">Cópias encriptadas da base de dados. Dados guardados em: <code class="text-xs">{dataDir}</code></p>
		</div>
		<button class="btn-primary" onclick={backupNow}>Fazer backup agora</button>
	</header>
	{#if backups.length === 0}
		<p class="muted">Ainda não há backups.</p>
	{:else}
		<div class="overflow-x-auto">
		<table class="table-base">
			<thead><tr><th>Ficheiro</th><th>Data</th><th class="text-right">Tamanho</th><th></th></tr></thead>
			<tbody>
				{#each backups as backup (backup.path)}
					<tr>
						<td class="font-mono text-xs">{backup.name}</td>
						<td>{formatDate(backup.createdAt)} {backup.createdAt.slice(11, 16)}</td>
						<td class="text-right">{sizeLabel(backup.size)}</td>
						<td class="text-right whitespace-nowrap">
							<button class="btn-ghost" onclick={() => verify(backup)}>Verificar</button>
							{#if isTauri}<button class="btn-ghost" onclick={() => restore(backup)}>Restaurar</button>{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
		</div>
	{/if}
</section>

<section class="card mt-6 grid gap-6 md:grid-cols-2">
	<div class="space-y-3">
		<h2 class="font-medium">Exportar tudo</h2>
		<p class="muted">Ficheiro versionado com todas as contas, movimentos, recorrências, créditos, objetivos e definições. Serve para migrar para outra máquina.</p>
		<label class="label">
			Password da exportação (recomendado)
			<input type="password" bind:value={exportPassword} class="input" autocomplete="new-password" />
		</label>
		<button class="btn-secondary" onclick={exportAll} disabled={busy}>{busy ? 'A processar…' : 'Exportar…'}</button>
	</div>
	<div class="space-y-3">
		<h2 class="font-medium">Importar</h2>
		<p class="muted">Valida o ficheiro e a versão antes de importar. Antes de importar é criado um backup automático.</p>
		<button class="btn-secondary" onclick={chooseImport}>Escolher ficheiro…</button>
	</div>
</section>

{#if isTauri}
	<form onsubmit={updatePassword} class="card mt-6 grid grid-cols-2 gap-4 md:grid-cols-4">
		<h2 class="col-span-2 font-medium md:col-span-4">Alterar password</h2>
		<label class="label">
			Password atual
			<input type="password" bind:value={passwords.current} class="input" autocomplete="current-password" required />
		</label>
		<label class="label">
			Nova password
			<input type="password" bind:value={passwords.next} minlength="8" class="input" autocomplete="new-password" required />
		</label>
		<label class="label">
			Confirmar
			<input type="password" bind:value={passwords.confirm} minlength="8" class="input" autocomplete="new-password" required />
		</label>
		<div class="flex items-end"><button class="btn-secondary w-full">Alterar</button></div>
		<p class="col-span-2 muted md:col-span-4">Não existe recuperação: se esqueceres a password, os dados ficam inacessíveis.</p>
	</form>
{/if}

{#if importing}
	<Modal title="Importar dados" onclose={() => (importing = null)}>
		<div class="space-y-4">
			<p class="font-mono text-xs break-all">{importing.path}</p>
			<label class="label">
				Password do ficheiro (se estiver encriptado)
				<input type="password" bind:value={importing.password} class="input" />
			</label>
			<button class="btn-secondary" onclick={preview} disabled={busy}>{busy ? 'A processar…' : 'Validar ficheiro'}</button>
			{#if importing.preview}
				<div class="rounded-md bg-stone-100 p-3 text-sm dark:bg-stone-800">
					<p>Formato v{importing.preview.formatVersion} · esquema {importing.preview.schemaVersion} · {importing.preview.encrypted ? 'encriptado' : 'não encriptado'}</p>
					<p>Exportado em {formatDate(importing.preview.exportedAt)}</p>
					<ul class="mt-2 grid grid-cols-2 gap-x-4 text-xs">
						{#each importing.preview.counts.filter((c) => c.rows > 0) as count (count.table)}<li>{count.table}: {count.rows}</li>{/each}
					</ul>
					<p class="mt-2">{importing.preview.conflicts} registos já existem (conflitos).</p>
				</div>
				<div class="flex justify-end gap-2">
					<button class="btn-secondary" disabled={busy} onclick={() => runImport(false)}>Juntar (ignora conflitos)</button>
					<button class="btn-danger" disabled={busy} onclick={() => runImport(true)}>Substituir tudo</button>
				</div>
			{/if}
		</div>
	</Modal>
{/if}
