# Blob Pop 🫧

Un jeu de puzzle arcade dynamique et rétro inspiré de l'esthétique Game Boy LCD. Visez, lancez et fusionnez des blobs gélatineux pour cumuler un maximum de points !

## 🎮 Fonctionnalités

- **Physique gélatineuse & collisions** : Les blobs se déforment et oscillent lors des impacts.
- **Rendu Rétro Game Boy** : Effet d'écran LCD vert à 3 nuances géré par un shader GLSL personnalisé.
- **Synthèse audio procédurale** : Effets sonores 8-bit (lancement, sélection, fusions) générés dynamiquement au format WAV en mémoire sans fichiers audio externes.
- **Cross-platform** : Fonctionne sur Desktop (Windows/Linux/macOS) et Android.

---

## 🛠️ Pile Technique

- **Langage** : [Rust](https://www.rust-lang.org/) (Édition 2021)
- **Moteur / Framework** : [Macroquad](https://macroquad.rs/) / Miniquad
- **Compilateur Android** : Container Docker officiel `notfl3/cargo-apk` avec SDK Android 33

---

## 💻 Compilation & Exécution

### 1. Sur PC (Desktop)

Assurez-vous d'avoir Rust et Cargo installés, puis lancez :

```bash
cargo run --release
```

---

### 2. Sur Android (APK)

La méthode recommandée utilise Docker pour éviter la gestion manuelle du SDK/NDK Android et des dépendances Java.

#### PRÉREQUIS :

* [Docker Desktop](https://www.docker.com/products/docker-desktop/) installé et démarré.
* [ADB](https://developer.android.com/tools/adb) (Android Debug Bridge) configuré et débogage USB activé sur le smartphone.

#### Compilation de l'APK :

À la racine du projet, exécutez dans votre terminal (PowerShell / Bash) :

```powershell
docker run --rm -v "${PWD}:/root/src" -w /root/src notfl3/cargo-apk bash -c "yes | /opt/android-sdk-linux/tools/bin/sdkmanager --sdk_root=/opt/android-sdk-linux 'platforms;android-33' && cargo quad-apk build --release"
```

**Note** : Si le SDK Android 33 est déjà téléchargé dans le conteneur, vous pouvez utiliser la commande raccourcie :

```powershell
docker run --rm -v "${PWD}:/root/src" -w /root/src notfl3/cargo-apk cargo quad-apk build --release
```

L'APK compilé sera généré dans :
`target/android-artifacts/release/apk/blobpop.apk`

#### Installation et nettoyage sur téléphone :

Pour réinstaller proprement l'application sur le téléphone connecté en USB :

```powershell
adb uninstall fr.julientissier.blobpop
adb install -r target\android-artifacts\release\apk\blobpop.apk
```
