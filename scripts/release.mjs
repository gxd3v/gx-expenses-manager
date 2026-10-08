import { execFileSync, execSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';

const REPO = 'gxd3v/gx-expenses-manager';
const KEY = join(homedir(), '.tauri', 'expenses-manager.key');
const LINUX_IMAGE = 'gestor-de-despesas-linux';

const [notesPath, flag] = process.argv.slice(2);
if (!notesPath) {
	console.error('usage: node scripts/release.mjs <release-notes.txt> [--publish]');
	process.exit(1);
}

try {
	execSync('docker info', { stdio: 'ignore' });
} catch {
	console.error('Docker is not running: start Docker Desktop to build the Linux packages');
	process.exit(1);
}

const { version, productName } = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const notes = readFileSync(notesPath, 'utf8').trim();
const tag = `v${version}`;
const name = productName.replaceAll(' ', '');
const assets = { windows: `${name}_x64-setup.exe`, appimage: `${name}_amd64.AppImage`, deb: `${name}_amd64.deb` };
const out = join('release', tag);
const linuxOut = join(out, 'linux');
const env = { ...process.env, TAURI_SIGNING_PRIVATE_KEY: readFileSync(KEY, 'utf8'), TAURI_SIGNING_PRIVATE_KEY_PASSWORD: '' };

rmSync(out, { recursive: true, force: true });
mkdirSync(linuxOut, { recursive: true });

execSync('npm run tauri build -- --config src-tauri/tauri.release.conf.json', { stdio: 'inherit', env });

execFileSync('docker', ['build', '-q', '-t', LINUX_IMAGE, 'scripts/linux'], { stdio: 'inherit' });
execFileSync(
	'docker',
	[
		'run', '--rm',
		'-e', 'TAURI_SIGNING_PRIVATE_KEY',
		'-e', 'TAURI_SIGNING_PRIVATE_KEY_PASSWORD',
		'-v', `${resolve('.')}:/repo:ro`,
		'-v', `${resolve(linuxOut)}:/out`,
		'-v', `${LINUX_IMAGE}-cargo:/usr/local/cargo/registry`,
		'-v', `${LINUX_IMAGE}-target:/target`,
		LINUX_IMAGE
	],
	{ stdio: 'inherit', env }
);

const metadata = JSON.parse(execSync('cargo metadata --format-version 1 --no-deps --manifest-path src-tauri/Cargo.toml').toString());
const windowsBundle = join(metadata.target_directory, 'release', 'bundle', 'nsis', `${productName}_${version}_x64-setup.exe`);
const linuxBundle = (extension) => join(linuxOut, readdirSync(linuxOut).find((file) => file.endsWith(extension)));
const bundles = { windows: windowsBundle, appimage: linuxBundle('.AppImage'), deb: linuxBundle('.deb') };

for (const [kind, file] of Object.entries(bundles)) copyFileSync(file, join(out, assets[kind]));

const platform = (kind) => ({
	signature: readFileSync(`${bundles[kind]}.sig`, 'utf8'),
	url: `https://github.com/${REPO}/releases/download/${tag}/${assets[kind]}`
});
const latest = (kind) => `https://github.com/${REPO}/releases/latest/download/${assets[kind]}`;

writeFileSync(join(out, 'notes.txt'), notes);
writeFileSync(
	join(out, 'github.txt'),
	`${notes}

## Instalação
- Windows: [${assets.windows}](${latest('windows')}).
- Linux: [${assets.appimage}](${latest('appimage')}) ou [${assets.deb}](${latest('deb')}) (Debian, Ubuntu e derivadas).
- Em instalações existentes, a atualização é feita pela própria aplicação.
`
);
writeFileSync(
	join(out, 'latest.json'),
	JSON.stringify(
		{
			version,
			notes,
			pub_date: new Date().toISOString(),
			platforms: {
				'windows-x86_64': platform('windows'),
				'linux-x86_64': platform('appimage'),
				...(existsSync(`${bundles.deb}.sig`) && { 'linux-x86_64-deb': platform('deb') })
			}
		},
		null,
		2
	)
);
console.log(`release files ready in ${out}`);

if (flag === '--publish') {
	execFileSync('git', ['push', 'origin', 'HEAD:main'], { stdio: 'inherit' });
	execFileSync(
		'gh',
		[
			'release', 'create', tag,
			...Object.values(assets).map((asset) => join(out, asset)),
			join(out, 'latest.json'),
			'--repo', REPO, '--target', 'main', '--title', `${productName} ${version}`, '--notes-file', join(out, 'github.txt')
		],
		{ stdio: 'inherit' }
	);
}
