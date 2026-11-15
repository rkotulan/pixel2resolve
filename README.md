# pixel2resolve

Windows utilita překóduje zvukové stopy videa MP4 do AAC-LC (48 kHz). Obraz zkopíruje bez rekomprese. Výsledek `nazev.p2r.mp4` uloží vedle zdrojového souboru. Při opakovaném převodu vytvoří `nazev (2).p2r.mp4` a další očíslované kopie.

## Sestavení a spuštění

Na Windows x64 s Rustem a nástroji MSVC spusť `cargo build --release --locked`. Výsledný program najdeš v `target\release\pixel2resolve.exe`. Nainstaluj FFmpeg do `PATH` nebo zkopíruj `ffmpeg.exe` vedle `pixel2resolve.exe`. [FFmpeg pro Windows](https://ffmpeg.org/download.html#build-windows)

Spusť `pixel2resolve.exe` a vyber **Vybrat video MP4**. Výsledné video můžeš přidat do Media Poolu v DaVinci Resolve.

## Nabídka Průzkumníku

V okně `pixel2resolve.exe` klikni na **Přidat do nabídky Průzkumníku**. Registrace platí pro aktuálního uživatele. Pak klikni pravým tlačítkem na MP4 a vyber **Zobrazit další možnosti → pixel2resolve – převést zvuk v MP4**. Pokud složku přesuneš, otevři utilitu z nového místa a vyber **Používat tuto kopii v nabídce Průzkumníku**.

Z okna aplikace můžeš položku také odebrat. Pro automatizaci slouží přepínače `--register-menu` a `--unregister-menu`. Vlastní název nastavíš například příkazem `pixel2resolve.exe --register-menu "Nový název"`.

## Licence

Zdrojový kód pixel2resolve je dostupný pod licencí [MIT](LICENSE). Utilita spouští FFmpeg jako samostatný proces. FFmpeg se řídí vlastními licenčními podmínkami. [Licence FFmpegu](https://ffmpeg.org/legal.html)
