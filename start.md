# Objetivo

Atue como um Engenheiro de Software Sênior especialista em Linux, Rust, GTK4 e Wayland.

Preciso construir um gerenciador de área de transferência (clipboard manager) para Ubuntu/Linux, **inspirado no [Maccy](https://github.com/p0deje/Maccy)** (macOS) — use o repositório como referência de filosofia e UX (não de código, já que o Maccy é em Swift): histórico leve e rápido, busca instantânea, navegação 100% por teclado, pin de itens, e opção de colar sem formatação.

Nome do projeto: Lincy

# Ambiente-alvo

- Ubuntu, sessão **Wayland/GNOME por padrão**, compositor **Mutter**.
- Monitoramento passivo de clipboard e hotkeys globais têm restrições de segurança no Wayland — proponha a solução tecnicamente correta para o Mutter especificamente, e documente um plano B (ex: sessão X11/Xorg, ainda disponível no login do Ubuntu) caso algum recurso não esteja disponível na versão do compositor instalada.

# Stack Tecnológica Obrigatória (use sempre a versão estável mais recente disponível no momento — não fixe versões antigas de memória, rode `cargo add` / consulte o crates.io para pegar a mais atual)

- **Linguagem:** Rust, edition 2024 (padrão desde a 1.85), compilador estável mais recente (canal `stable` via `rustup`).
- **Interface Gráfica:** GTK4 + Libadwaita via `gtk-rs` (crates `gtk4` e `libadwaita`, série 0.11.x ou superior) — para integração visual nativa com GNOME/Ubuntu.
- **Clipboard:**
  - `wl-clipboard-rs` — monitoramento nativo via `watch`/data-control no Wayland (protocolo `ext-data-control-v1`, com fallback para `wlr-data-control` em compositores wlroots).
  - `arboard` — fallback simples de get/set, útil para sessão X11.
- **Hotkey global:** `ashpd` — wrapper idiomático para o portal `GlobalShortcuts` do `xdg-desktop-portal` (via D-Bus), forma suportada no GNOME/Wayland.
- **Banco de dados:** SQLite embutido via `rusqlite` (feature `bundled`), série 0.38.x ou superior.
- **Async runtime:** `tokio`, para o daemon em background e chamadas D-Bus/portal.

# Requisitos do MVP

1. **Daemon em Background:** roda silenciosamente, escuta mudanças na área de transferência e salva automaticamente o conteúdo (apenas texto puro inicialmente) no SQLite.
2. **Interface Pop-up (Search & Paste):** ao pressionar um atalho global (ex: `Super + Shift + C`), abre uma janela GTK4 simples e centralizada.
3. **Busca instantânea:** barra de pesquisa focada + lista do histórico recente; digitar filtra os resultados do SQLite em tempo real.
4. **Navegação e Ação:** setas do teclado para navegar; `Enter` copia o item selecionado de volta pro clipboard e fecha a janela.
5. **Pin de itens:** manter um item fixo no topo da lista, sem expirar no histórico (como o Maccy faz).
6. **Colar sem formatação:** variante do "Enter" que copia apenas o texto puro, sem metadados de formatação.

# Sua Tarefa Inicial

1. Confirme a versão estável mais recente do Rust e das crates principais (`gtk4`, `libadwaita`, `rusqlite`, `ashpd`, `wl-clipboard-rs`) antes de gerar o `Cargo.toml` — não assuma números de memória.
2. Forneça o comando `cargo new` correto e liste as dependências essenciais do `Cargo.toml`.
3. Crie a arquitetura de pastas inicial, separando: backend do banco de dados, monitor de clipboard em background (daemon), integração com o portal de hotkeys, e a interface GTK.
4. Escreva o código base do `main.rs` e do módulo `rusqlite` para inicializar a tabela de histórico de textos (colunas sugeridas: id, conteúdo, hash, pinned, created_at).
5. Explique a estratégia recomendada para registrar o atalho global no Rust/Wayland via `ashpd`/`GlobalShortcuts`, incluindo o plano B para quando o portal não estiver disponível.
6. Explique a estratégia de monitoramento de clipboard via `wl-clipboard-rs` (watch) vs. polling com `arboard`, e quando usar cada uma.
