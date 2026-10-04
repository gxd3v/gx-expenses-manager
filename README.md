# Expenses Manager

A personal finance app for your desktop. It keeps track of where your money goes and shows you where it's heading. It runs entirely on your own computer.

No account, no cloud, no subscription. Your data is one encrypted file, and only your password opens it.

## What it does

- **Accounts and transactions.** Track checking, savings, cash and cards. Record income, expenses and transfers between accounts, and organise them with categories and subcategories.
- **Recurring payments.** Add salary, rent, subscriptions and anything else that repeats. You can skip a single occurrence or change one, and confirmed payments turn into real transactions.
- **Loans and credits.** Get amortization tables and see how each payment splits into principal and interest. You can also simulate early repayments.
- **Goals.** Set savings targets and follow your progress.
- **Forecasts.** See where your balance will be in the coming months, with fixed and variable spending kept apart. "What if" scenarios let you test changes before you make them.
- **Monthly view and charts.** Compare what you planned with what actually happened. Charts break down spending by category, income against expenses, and net worth over time.
- **Day-to-day tools.**
  - Reconciliation
  - Templates
  - Quick add (`N`)
  - Global search (`Ctrl+K`)
  - Saved filters
  - CSV export
- **Alerts.** Get notified about upcoming payments, low balances and a negative forecast.
- **Privacy.**
  - Lock the app with `Ctrl+L`, or let it lock itself when idle.
  - Hide every amount on screen with `Ctrl+H`, handy when someone is looking over your shoulder.
- **Backups.** Automatic backups run in the background. You can also export a file and import it on another machine, with optional encryption.
- **Light and dark themes.**

## 100% offline

The app never connects to the internet.

- **No network calls.** The interface talks to the backend inside the same process. The app opens no ports and runs no local web server.
- **Network access is blocked.** The window's content security policy only allows that in-process channel, so nothing on the page can reach out.
- **No telemetry.** There are no analytics, no crash reporting and no update checks.
- **Your data stays local.** Everything lives in one SQLite database on your disk, encrypted with SQLCipher (AES-256).

The one exception is the installer. If your Windows doesn't have the Microsoft WebView2 runtime yet, the installer may download it. Windows 10 (recent updates) and Windows 11 already include it.

### About your password

Your password is never stored anywhere. The app turns it into the key that encrypts the database, and keeps it only in memory while the app is unlocked.

**If you forget your password, your data cannot be recovered.** Nobody can reset it, because there is nobody to ask. Choose a password you'll remember, and keep a backup.

## Installation (Windows)

1. Download the installer from the [Releases](https://github.com/gxd3v/gx-expenses-manager/releases) page:
   - `Expenses Manager_x.y.z_x64-setup.exe` (recommended). It installs for your user only and doesn't need admin rights.
   - `Expenses Manager_x.y.z_x64_en-US.msi` is available if you prefer an MSI.
2. Run it. The installer isn't code-signed, so Windows SmartScreen may warn you. Click **More info → Run anyway**.
3. Open **Expenses Manager** from the Start menu.
4. On first launch, create your password (at least 8 characters). That's it.

### Updating

Run the newer installer over the existing installation. Your data is kept, and the database is backed up automatically before any upgrade that changes its structure.

### Uninstalling

Use **Settings → Apps → Expenses Manager → Uninstall**. Your data is **kept** by default, so reinstalling picks up where you left off.

To wipe everything, tick **"Delete the application data"** in the uninstaller.

### Where your data lives

| What | Where |
|---|---|
| Database | `%APPDATA%\com.gxd3v.expenses\expenses.db` |
| Backups (default) | `%APPDATA%\com.gxd3v.expenses\backups\` |
| Logs (errors only, no amounts) | `%LOCALAPPDATA%\com.gxd3v.expenses\logs\` |

You can change the backup folder in Settings. A folder you sync yourself is fine, because the backup files are encrypted too.

## Building from source

You'll need:

- Windows 10/11
- [Node.js](https://nodejs.org) 22.17 or newer
- [Rust](https://rustup.rs) (MSVC toolchain) and the Visual Studio C++ Build Tools
- [Strawberry Perl](https://strawberryperl.com). The build compiles OpenSSL for SQLCipher, and the Perl that ships with Git doesn't work.

```bash
git clone https://github.com/gxd3v/gx-expenses-manager.git
cd gx-expenses-manager
npm install
npm run tauri build
```

The installers end up in `src-tauri/target/release/bundle/` (`nsis/` and `msi/`).

> If the OpenSSL step fails with path-length errors, point Cargo to a shorter target directory, for example `CARGO_TARGET_DIR=C:\cx`.

### Development

```bash
npm run tauri dev                 # run the app with hot reload (uses a separate dev database)
cd src-tauri && cargo test        # backend tests
```

## Built with

[Tauri 2](https://tauri.app) · Rust · [Svelte 5](https://svelte.dev) · Tailwind CSS 4 · GraphQL ([async-graphql](https://github.com/async-graphql/async-graphql)) · SQLite + [SQLCipher](https://www.zetetic.net/sqlcipher/)
