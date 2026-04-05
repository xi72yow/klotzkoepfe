# Klotzkoepfe

Klotzkoepfe ist ein rasanter 2D-Top-Down-Zombie-Survival-Shooter fuer bis zu zwei Spieler an einer Tastatur. Welle um Welle stuermen immer staerkere Zombies auf euch ein, waehrend ihr durch das Zerstoeren von Kisten neue Waffen wie Schrotflinten, Granaten, Raketen und Minen freischaltet. Taktisches Zusammenspiel, geschicktes Positionieren und der gezielte Einsatz der verschiedenen Waffen entscheiden ueber euer Ueberleben. Wer den hoechsten Combo-Multiplikator aufbaut und die meisten Wellen uebersteht, gewinnt.

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

### Manuell bauen

```bash
# Abhaengigkeiten (Debian/Ubuntu)
sudo apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev

# Build & Starten
cargo build --release
./target/release/klotzkoepfe
```
