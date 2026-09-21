# Cenda: Security Auditor

Small, read-only Linux security configuration auditor written in Rust.

`cenda` revisa un conjunto reducido de configuraciones básicas de seguridad de un sistema Linux y genera un reporte legible para humanos o en formato JSON.

El proyecto está pensado como una herramienta educativa y práctica para combinar:

- Desarrollo de CLI en Rust.
- Lectura y parsing de archivos Linux.
- Inspección de permisos Unix.
- Manejo de errores.
- Testing.
- Serialización JSON.
- Automatización y posibles usos en CI/CD.

> `cenda` no pretende realizar una evaluación completa de seguridad. Un resultado `PASS` únicamente significa que una comprobación específica fue satisfactoria.

## Features

La primera versión incluye:

- Análisis básico de `/etc/passwd`.
- Detección de entradas malformadas.
- Detección de cuentas adicionales con UID `0`.
- Conteo de usuarios con shell potencialmente interactiva.
- Revisión de permisos de `/etc/passwd`.
- Revisión de permisos de `/etc/group`.
- Revisión de permisos de `/etc/shadow`.
- Análisis explícito de algunas directivas de `sshd_config`.
- Salida legible para humanos.
- Salida JSON.
- Códigos de salida adecuados para automatización.
- Tests unitarios y de integración.
- Funcionamiento de solo lectura.

## Alcance

En la versión inicial se revisan tres áreas:

1. SSH.
2. Usuarios.
3. Permisos de archivos críticos.

No se incluyen:

- Network scanning.
- Vulnerability scanning.
- Bases de datos de CVEs.
- Captura de paquetes.
- Detección de malware.
- Análisis de procesos.
- Auditoría de Docker.
- Análisis completo del firewall.
- Corrección automática.
- Modificación de archivos del sistema.
- Servicio en segundo plano.
- Interfaz web.
- Base de datos.
- Auditoría remota.

## Requisitos

- Linux.
- Rust y Cargo.
- Permisos suficientes para inspeccionar los archivos disponibles.

Algunas comprobaciones, especialmente la lectura de `/etc/shadow`, pueden requerir privilegios elevados.

## Instalación desde el código fuente

```bash
git clone <repository-url>
cd cenda
cargo build --release
```

El binario compilado estará disponible en:

```text
target/release/cenda
```

También puede instalarse localmente mediante Cargo:

```bash
cargo install --path .
```

## Uso

Ejecutar una auditoría:

```bash
cenda
```

Mostrar ayuda:

```bash
cenda --help
```

Mostrar la versión:

```bash
cenda --version
```

Generar salida JSON:

```bash
cenda --json
```

Activar información adicional:

```bash
cenda --verbose
```

## Ejemplo de salida humana

```text
Linux Security Audit
====================

[USERS]
  INFO  Users found: 12
  INFO  Interactive shell users: 2
  PASS  No additional UID 0 accounts detected
  PASS  No malformed /etc/passwd entries detected

[SSH]
  PASS  SSH root login is explicitly disabled
  WARN  SSH password authentication is enabled

[PERMISSIONS]
  PASS  /etc/passwd is not writable by group or other users
  PASS  /etc/group is not writable by group or other users
  PASS  /etc/shadow is not world-readable

Summary
-------
PASS: 5
WARN: 1
INFO: 2
ERROR: 0
```

## Ejemplo de salida JSON

```json
{
  "tool": "cenda",
  "version": "0.1.0",
  "schema_version": 1,
  "checks": [
    {
      "id": "ssh_password_authentication",
      "category": "ssh",
      "status": "warn",
      "message": "SSH password authentication is enabled",
      "recommendation": "Consider disabling password authentication and using key-based authentication"
    },
    {
      "id": "passwd_additional_uid_0",
      "category": "users",
      "status": "pass",
      "message": "No additional UID 0 accounts detected",
      "recommendation": null
    }
  ],
  "summary": {
    "pass": 1,
    "warn": 1,
    "info": 0,
    "error": 0
  }
}
```

## Códigos de salida

`cenda` utiliza los siguientes códigos:

```text
0 = La auditoría terminó sin warnings
1 = Se encontraron uno o más warnings
2 = Ocurrió un error fatal durante la ejecución
```

Los errores de comprobaciones individuales no deberían detener toda la auditoría cuando sea posible continuar con los demás checks.

Por ejemplo, si `/etc/shadow` no puede leerse:

```text
WARN  Could not inspect /etc/shadow: permission denied
```

La herramienta debería intentar continuar con las demás comprobaciones.

## Tests

Ejecutar todos los tests:

```bash
cargo test
```

Ejecutar los tests con salida detallada:

```bash
cargo test -- --nocapture
```

Comprobar el código con Clippy:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Formatear el código:

```bash
cargo fmt
```

## Seguridad y permisos

La herramienta funciona en modo de solo lectura.

No realiza ninguna de las siguientes acciones:

- No modifica `sshd_config`.
- No cambia permisos.
- No crea usuarios.
- No elimina usuarios.
- No instala paquetes.
- No ejecuta `sudo` automáticamente.
- No aplica remediaciones.
- No abre ni cierra puertos.

El objetivo es observar y reportar.

## Limitaciones

La primera versión utiliza reglas deliberadamente simples.

Por ejemplo:

- La configuración efectiva de OpenSSH puede depender de valores por defecto.
- `Include` puede cargar otros archivos de configuración.
- Las reglas de `sshd_config` no representan toda la semántica de OpenSSH.
- La detección de shells interactivas](#)
