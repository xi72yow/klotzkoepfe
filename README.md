# Klotzkoepfe

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
