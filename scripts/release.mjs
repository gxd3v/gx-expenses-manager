import { execFileSync, execSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const REPO = 'gxd3v/gx-expenses-manager';
const KEY = join(homedir(), '.tauri', 'expenses-manager.key');

const [notesPath, flag] = process.argv.slice(2);
if (!notesPath) {
	console.error('usage: node scripts/release.mjs <release-notes.txt> [--publish]');
	process.exit(1);
}

const { version, productName } = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const notes = readFileSync(notesPath, 'utf8').trim();
const tag = `v${version}`;
const installer = `${productName.replaceAll(' ', '')}_${version}_x64-setup.exe`;
const out = join('release', tag);

execSync('npm run tauri build -- --config src-tauri/tauri.release.conf.json', {
	stdio: 'inherit',
	env: { ...process.env, TAURI_SIGNING_PRIVATE_KEY: readFileSync(KEY, 'utf8'), TAURI_SIGNING_PRIVATE_KEY_PASSWORD: '' }
});

const metadata = JSON.parse(execSync('cargo metadata --format-version 1 --no-deps --manifest-path src-tauri/Cargo.toml').toString());
const bundle = join(metadata.target_directory, 'release', 'bundle', 'nsis', `${productName}_${version}_x64-setup.exe`);

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
copyFileSync(bundle, join(out, installer));
writeFileSync(join(out, 'notes.txt'), notes);
writeFileSync(
	join(out, 'latest.json'),
	JSON.stringify(
		{
			version,
			notes,
			pub_date: new Date().toISOString(),
			platforms: {
				'windows-x86_64': {
					signature: readFileSync(`${bundle}.sig`, 'utf8'),
					url: `https://github.com/${REPO}/releases/download/${tag}/${installer}`
				}
			}
		},
		null,
		2
	)
);
console.log(`release files ready in ${out}`);

if (flag === '--publish') {
	if (!existsSync(join(out, 'latest.json'))) process.exit(1);
	execFileSync('git', ['push', 'origin', 'HEAD:main'], { stdio: 'inherit' });
	execFileSync(
		'gh',
		['release', 'create', tag, join(out, installer), join(out, 'latest.json'), '--repo', REPO, '--target', 'main', '--title', `${productName} ${version}`, '--notes-file', join(out, 'notes.txt')],
		{ stdio: 'inherit' }
	);
}
