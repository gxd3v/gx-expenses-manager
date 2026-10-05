import { confirmAction } from './dialogs.ts';
import { dataChanged } from './refs.svelte.ts';
import { startSimulation, stopSimulation } from './session.ts';
import { loadSettings } from './settings.svelte.ts';
import { notify, notifyError } from './toasts.svelte.ts';

export const simulation = $state({ active: false, busy: false });

export async function toggleSimulation() {
	if (simulation.busy) return;
	const ending = simulation.active;
	if (ending && !(await confirmAction('Terminar a simulação? Todas as alterações feitas durante a simulação são descartadas.'))) return;
	simulation.busy = true;
	try {
		await (ending ? stopSimulation() : startSimulation());
		simulation.active = !ending;
		await loadSettings();
		await dataChanged();
		notify(ending ? 'Simulação terminada. Dados reais repostos.' : 'Modo simulação ativo. As alterações não são guardadas.');
	} catch (e) {
		notifyError(e);
	} finally {
		simulation.busy = false;
	}
}
