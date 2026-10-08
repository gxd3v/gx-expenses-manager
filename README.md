# Gestor de Despesas

Aplicação de finanças pessoais para computador (Windows e Linux). Regista para onde vai o dinheiro e mostra para onde caminha o saldo. Funciona inteiramente no próprio computador.

Sem conta, sem cloud, sem subscrição. Os dados ficam num único ficheiro encriptado, que só a password abre.

## Funcionalidades

- **Contas e movimentos.** Contas à ordem, poupança, cartões, cartão refeição e dinheiro físico. Receitas, despesas e transferências entre contas, organizadas por categorias e subcategorias.
- **Recorrências.** Salário, renda, subscrições e transferências periódicas. Cada ocorrência pode ser saltada ou alterada, e os valores variáveis ficam a aguardar confirmação.
- **Créditos.** Plano de amortização, divisão de cada prestação em capital e juros, e simulação de amortizações antecipadas.
- **Contas com juros.** Taxa fixa ou por escalões e periodicidade do vencimento, com os juros líquidos incluídos nas previsões.
- **Objetivos.** Metas de poupança com acompanhamento do progresso.
- **Modo simulação.** Movimentos, transferências e recorrências virtuais sobre uma cópia temporária dos dados, para ver o efeito nos saldos e nas previsões. Ao terminar, a cópia é descartada.
- **Compras planeadas.** Simulação da data mais cedo para uma compra e lista de desejos com datas previstas.
- **Previsões.** Saldo previsto para os próximos meses, com gastos fixos e variáveis separados e cenários hipotéticos. Movimentos pontuais ficam fora das médias.
- **Vista mensal e gráficos.** Comparação entre o previsto e o realizado, gastos por categoria, receitas contra despesas, evolução do património e máximos e mínimos de saldo.
- **Ferramentas do dia a dia.**
  - Reconciliação
  - Templates
  - Adição rápida (`N`)
  - Pesquisa global (`Ctrl+K`)
  - Filtros guardados
  - Exportação CSV
- **Avisos.** Pagamentos próximos, agendamentos a terminar, saldo baixo e previsão negativa.
- **Privacidade.**
  - Bloqueio com `Ctrl+L` ou automático por inatividade.
  - Ocultação de todos os valores no ecrã com `Ctrl+H`.
- **Backups.** Backups automáticos e exportação para outro computador, com encriptação opcional.
- **Tema claro e escuro.**

## Funcionamento offline

Os dados nunca saem do computador.

- **Nenhum dado é enviado.** A interface comunica com o backend dentro do mesmo processo. A aplicação não abre portas nem corre um servidor web local.
- **A interface não acede à internet.** A política de segurança de conteúdo da janela só permite esse canal interno.
- **Sem telemetria.** Não há estatísticas de utilização nem relatórios de erros.
- **Dados locais.** Tudo fica numa base de dados SQLite no disco, encriptada com SQLCipher (AES-256).

Existe um único pedido de rede: a **verificação de atualizações**. Com a aplicação desbloqueada, é descarregado um pequeno ficheiro público (`latest.json`) das releases deste repositório para saber se existe uma versão nova. Nenhuma informação pessoal ou financeira é enviada. A verificação pode ser desligada em **Definições → Atualizações**, e a partir daí a aplicação não faz qualquer pedido de rede.

No Windows, o instalador pode ainda descarregar o Microsoft WebView2, caso não esteja instalado. O Windows 11 e o Windows 10 atualizado já o incluem.

### Password

A password não é guardada em lado nenhum. A aplicação deriva dela a chave que encripta a base de dados e mantém-na apenas em memória enquanto está desbloqueada.

**Sem a password, os dados não podem ser recuperados.** Não existe forma de a repor. Recomenda-se guardá-la num gestor de passwords e manter backups.

## Instalação (Windows)

1. Descarregar o instalador da versão mais recente: **[GestordeDespesas_x64-setup.exe](https://github.com/gxd3v/gx-expenses-manager/releases/latest/download/GestordeDespesas_x64-setup.exe)**. Cada [release](https://github.com/gxd3v/gx-expenses-manager/releases) inclui sempre este ficheiro. Instala apenas para o utilizador atual e não precisa de permissões de administrador.
2. Executar o instalador. Não tem assinatura de código, por isso o Windows SmartScreen pode mostrar um aviso: **Mais informações → Executar mesmo assim**.
3. Abrir o **Gestor de Despesas** a partir do menu Iniciar.
4. No primeiro arranque, definir a password (mínimo de 8 caracteres).

## Instalação (Linux)

Requer uma distribuição de 64 bits recente, como o Ubuntu 22.04, o Debian 12 ou o Fedora 38, ou posterior.

**AppImage** (qualquer distribuição):

1. Descarregar **[GestordeDespesas_amd64.AppImage](https://github.com/gxd3v/gx-expenses-manager/releases/latest/download/GestordeDespesas_amd64.AppImage)**.
2. Dar permissão de execução: `chmod +x GestordeDespesas_amd64.AppImage`.
3. Executar o ficheiro. Em algumas distribuições é preciso instalar o `libfuse2` (no Ubuntu: `sudo apt install libfuse2`).

**Pacote .deb** (Debian, Ubuntu e derivadas):

1. Descarregar **[GestordeDespesas_amd64.deb](https://github.com/gxd3v/gx-expenses-manager/releases/latest/download/GestordeDespesas_amd64.deb)**.
2. Instalar com `sudo apt install ./GestordeDespesas_amd64.deb`.
3. Abrir o **Gestor de Despesas** a partir do menu de aplicações.

## Atualizações

Quando existe uma versão nova, aparece uma barra no topo da aplicação. **Ver novidades** mostra as notas da versão e **Atualizar agora** descarrega a versão nova, verifica a assinatura, instala e reabre a aplicação. Os dados mantêm-se, e a base de dados é copiada automaticamente antes de qualquer atualização que altere a sua estrutura.

Também é possível atualizar manualmente, instalando a versão mais recente sobre a existente. No Linux, com o pacote .deb, a atualização pede a password de administrador.

## Desinstalação

- **Windows:** em **Definições do Windows → Aplicações → Gestor de Despesas → Desinstalar**. Para apagar tudo, basta marcar no desinstalador a opção de eliminar os dados da aplicação.
- **Linux:** apagar o ficheiro AppImage, ou `sudo apt remove gestor-de-despesas` no caso do pacote .deb. A pasta dos dados, indicada abaixo, pode ser apagada à mão.

Os dados são **mantidos** por omissão, pelo que uma reinstalação retoma o estado anterior.

## Localização dos dados

| O quê | Windows | Linux |
|---|---|---|
| Base de dados | `%APPDATA%\com.gxd3v.expenses\expenses.db` | `~/.local/share/com.gxd3v.expenses/expenses.db` |
| Backups (por omissão) | `%APPDATA%\com.gxd3v.expenses\backups\` | `~/.local/share/com.gxd3v.expenses/backups/` |
| Registos (apenas erros, sem valores) | `%LOCALAPPDATA%\com.gxd3v.expenses\logs\` | `~/.local/share/com.gxd3v.expenses/logs/` |

A pasta dos backups pode ser alterada nas Definições. Uma pasta sincronizada com a cloud é segura, porque os backups também estão encriptados. Para passar os dados de um sistema para o outro, basta exportar um backup num e importá-lo no outro.

## Compilar a partir do código

Requisitos comuns: [Node.js](https://nodejs.org) 22.17 ou superior e [Rust](https://rustup.rs).

- **Windows 10/11:** toolchain MSVC do Rust, Visual Studio C++ Build Tools e [Strawberry Perl](https://strawberryperl.com). A compilação gera o OpenSSL usado pelo SQLCipher, e o Perl incluído no Git não serve.
- **Linux:** as [dependências de sistema do Tauri](https://v2.tauri.app/start/prerequisites/#linux). No Ubuntu: `sudo apt install build-essential curl file perl pkg-config libssl-dev libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev`.

```bash
git clone https://github.com/gxd3v/gx-expenses-manager.git
cd gx-expenses-manager
npm install
npm run tauri build
```

Os pacotes ficam em `src-tauri/target/release/bundle/`: `nsis/` e `msi/` no Windows, `appimage/` e `deb/` no Linux.

> No Windows, se a compilação do OpenSSL falhar por caminhos demasiado longos, basta apontar o Cargo para uma pasta mais curta, por exemplo `CARGO_TARGET_DIR=C:\cx`.

As releases são geradas a partir do Windows com `node scripts/release.mjs <notas.txt>`, que compila também os pacotes Linux num contentor Docker (`scripts/linux/`). Para isso, o Docker Desktop tem de estar a correr.

### Desenvolvimento

```bash
npm run tauri dev                 # aplicação com recarregamento automático (base de dados de desenvolvimento separada)
cd src-tauri && cargo test        # testes do backend
```

## Tecnologias

[Tauri 2](https://tauri.app) · Rust · [Svelte 5](https://svelte.dev) · Tailwind CSS 4 · GraphQL ([async-graphql](https://github.com/async-graphql/async-graphql)) · SQLite + [SQLCipher](https://www.zetetic.net/sqlcipher/)
