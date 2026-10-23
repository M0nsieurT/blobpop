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
- **Compilateur Android** : Container Docker officiel `notfl3/cargo-apk`

---

## 💻 Compilation & Exécution

### 1. Sur PC (Desktop)

Assurez-vous d'avoir Rust et Cargo installés, puis lancez :

```bash
cargo run --release
```

---

### 2. Sur Android (APK)

La méthode recommandée utilise Docker pour éviter la gestion manuelle du SDK/NDK Android et des versions de Java.

#### PRÉREQUIS :

* [Docker Desktop](https://www.docker.com/products/docker-desktop/) installé et démarré.
* [ADB](https://developer.android.com/tools/adb) (Android Debug Bridge) pour l'installation sur appareil.

#### Compilation de l'APK :

À la racine du projet, exécutez :

```powershell
docker run --rm -v "${PWD}:/root/src" -w /root/src notfl3/cargo-apk cargo quad-apk build --release
```

L'APK compilé sera généré dans :
`target/android-artifacts/release/apk/blobpop.apk`

#### Installation sur téléphone :

Connectez votre smartphone Android (avec le débogage USB activé) et lancez :

```powershell
adb install -r target/android-artifacts/release/apk/blobpop.apk
```