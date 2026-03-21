# Klotzkoepfe

Boxhead-artiger 2-Spieler Zombie-Shooter (Bevy 0.18, Rust).

2 Spieler auf einer Tastatur: P1 WASD+Space+Q, P2 Pfeiltasten+Enter+RShift.

## Installation (Debian/Ubuntu)

### APT Repository

```bash
# GPG Key hinzufuegen
curl -fsSL https://xi72yow.github.io/klotzkoepfe/key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/klotzkoepfe.gpg

# Repository hinzufuegen
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/klotzkoepfe.gpg] https://xi72yow.github.io/klotzkoepfe stable main" \
  | sudo tee /etc/apt/sources.list.d/klotzkoepfe.list

# Installieren
sudo apt update && sudo apt install klotzkoepfe
```

### Update

```bash
sudo apt update && sudo apt upgrade klotzkoepfe
```

## Manuell bauen

### Abhaengigkeiten (Debian/Ubuntu)

```bash
sudo apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

### Build

```bash
cargo build --release
./target/release/klotzkoepfe
```

## Steuerung

| Aktion | Spieler 1 | Spieler 2 |
|--------|-----------|-----------|
| Bewegen | WASD | Pfeiltasten |
| Schiessen | Space | Enter |
| Waffe wechseln | Q | RShift |
| Pause/Settings | ESC | ESC |

### Pause-Menue

- **Up/Down**: Wert waehlen
- **Left/Right**: Wert aendern (Shift: 10x)
- **Tab/Shift+Tab**: Kategorie wechseln
- **F5**: Settings speichern
- **F6**: Defaults wiederherstellen

## Release erstellen

Ein neues Release wird ueber GitHub Actions erstellt:

1. Actions > CI > Run workflow
2. `release: yes` setzen
3. Optional: Version angeben (sonst aus Cargo.toml)
