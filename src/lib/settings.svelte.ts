import { gql } from './graphql.ts';

export type DateFormat = 'DAY_MONTH_YEAR' | 'YEAR_MONTH_DAY' | 'MONTH_DAY_YEAR';
export type Theme = 'SYSTEM' | 'LIGHT' | 'DARK';
export type ForecastMethod = 'RECURRING' | 'HISTORY';

export type Settings = {
	currency: string;
	dateFormat: DateFormat;
	firstDayOfWeek: number;
	theme: Theme;
	lockTimeoutMinutes: number;
	backupDir: string | null;
	backupFrequencyDays: number;
	backupKeep: number;
	forecastMethod: ForecastMethod;
	forecastHistoryMonths: number;
	forecastHorizonMonths: number;
	notificationsEnabled: boolean;
	notifyDaysAhead: number;
	notifyUpcoming: boolean;
	notifyCredits: boolean;
	notifyGoals: boolean;
	notifyLowBalance: boolean;
	lowBalanceThreshold: number;
	notifyNegativeForecast: boolean;
	checkUpdates: boolean;
};

const fields = `currency dateFormat firstDayOfWeek theme lockTimeoutMinutes backupDir backupFrequencyDays backupKeep
	forecastMethod forecastHistoryMonths forecastHorizonMonths notificationsEnabled notifyDaysAhead notifyUpcoming
	notifyCredits notifyGoals notifyLowBalance lowBalanceThreshold notifyNegativeForecast checkUpdates`;

export const app = $state<{ settings: Settings | null }>({ settings: null });

export async function loadSettings(): Promise<Settings> {
	const data = await gql<{ settings: Settings }>(`{ settings { ${fields} } }`);
	app.settings = data.settings;
	applyTheme(data.settings.theme);
	return data.settings;
}

export async function saveSettings(input: Settings): Promise<Settings> {
	const data = await gql<{ updateSettings: Settings }>(
		`mutation ($input: SettingsInput!) { updateSettings(input: $input) { ${fields} } }`,
		{ input }
	);
	app.settings = data.updateSettings;
	applyTheme(data.updateSettings.theme);
	return data.updateSettings;
}

const THEME_KEY = 'expenses-manager-theme';

export function applyTheme(theme: Theme) {
	const dark = theme === 'DARK' || (theme === 'SYSTEM' && matchMedia('(prefers-color-scheme: dark)').matches);
	document.documentElement.classList.toggle('dark', dark);
	try {
		localStorage.setItem(THEME_KEY, theme);
	} catch {
		return;
	}
}

export function restoreTheme() {
	try {
		applyTheme((localStorage.getItem(THEME_KEY) as Theme | null) ?? 'SYSTEM');
	} catch {
		applyTheme('SYSTEM');
	}
}
