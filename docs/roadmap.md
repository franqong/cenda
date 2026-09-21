# Roadmap de Cenda

## Objetivo

Construir una herramienta pequeña, correcta y testeable para auditar configuraciones básicas de seguridad de Linux.

El roadmap está dividido en iteraciones pequeñas. Cada etapa debe producir una mejora ejecutable y verificable.

## MVP 0.1 — Parser de usuarios

### Objetivo

Implementar la primera funcionalidad real sin depender todavía de SSH, permisos o JSON.

### Funcionalidades

- Crear el proyecto Rust.
- Leer `/etc/passwd`.
- Parsear sus entradas.
- Contar usuarios.
- Contar cuentas con UID `0`.
- Detectar entradas malformadas.
- Detectar shells potencialmente interactivas.
- Mostrar resultados básicos.

### Salida esperada

```text
Linux Security Audit

[USERS]
  INFO  Users found: 12
  INFO  Interactive shell users: 2
  INFO  UID 0 accounts: 1
  PASS  No malformed entries detected
```

### Tests

- Entrada válida.
- Entrada vacía.
- Entrada con campos insuficientes.
- UID no numérico.
- GID no numérico.
- Múltiples cuentas con UID `0`.
- Usuarios con shells conocidas.
- Usuarios con `/usr/sbin/nologin`.
- Líneas vacías.

### Criterio de finalización

El parser funciona con strings de prueba y la aplicación puede generar un reporte básico desde `/etc/passwd`.

---

## MVP 0.2 — Permisos de archivos

### Objetivo

Agregar inspección de metadata y permisos Unix.

### Funcionalidades

Revisar:

```text
/etc/passwd
/etc/group
/etc/shadow
```

Reglas iniciales:

- `/etc/shadow` no debe ser legible por others.
- `/etc/passwd` no debe ser escribible por group u others.
- `/etc/group` no debe ser escribible por group u others.

### Salida esperada

```text
[PERMISSIONS]
  PASS  /etc/passwd is not writable by group or other users
  PASS  /etc/group is not writable by group or other users
  PASS  /etc/shadow is not world-readable
```

### Manejo de errores

Si `/etc/shadow` no puede leerse:

```text
WARN  Could not inspect /etc/shadow: permission denied
```

El auditor debe continuar con los demás checks.

### Tests

- Archivo con permisos esperados.
- Archivo legible por others.
- Archivo escribible por group.
- Archivo escribible por others.
- Archivo inexistente.
- Error de permisos.
- Inspección de owner, group y mode bits.

### Criterio de finalización

Los checks funcionan sin modificar permisos y pueden probarse mediante archivos temporales.

---

## MVP 0.3 — Configuración SSH

### Objetivo

Agregar el análisis explícito de `/etc/ssh/sshd_config`.

### Funcionalidades

Detectar:

```text
PermitRootLogin no
PermitRootLogin yes
PasswordAuthentication no
PasswordAuthentication yes
```

También reconocer:

```text
PermitRootLogin prohibit-password
PermitRootLogin without-password
```

### Salida esperada

```text
[SSH]
  PASS  SSH root login is explicitly disabled
  WARN  SSH password authentication is enabled
```

### Casos no determinados

Si no se encuentra una directiva explícita:

```text
INFO  Could not determine effective PermitRootLogin setting
```

Si el archivo no existe:

```text
INFO  SSH configuration file was not found
```

### Limitaciones iniciales

La primera versión:

- No resolverá completamente `Include`.
- No calculará todos los defaults de OpenSSH.
- No garantizará cuál es la configuración efectiva del daemon.
- No ejecutará `sshd -T` automáticamente.

Estas limitaciones deben estar documentadas.

### Tests

- Directivas con espacios simples.
- Directivas con múltiples espacios.
- Directivas separadas por tabulaciones.
- Comentarios.
- Directivas repetidas.
- Valores desconocidos.
- Ausencia de directivas.
- Archivo inexistente.
- Configuración con `Include`.

### Criterio de finalización

El parser puede diferenciar estados explícitos y desconocidos sin inventar conclusiones.

---

## v0.4 — Modelo común de resultados

### Objetivo

Unificar todos los checks bajo un mismo modelo de datos.

### Funcionalidades

Crear:

```rust
enum Status {
    Pass,
    Warn,
    Info,
    Error,
}
```

```rust
enum Category {
    Ssh,
    Users,
    Permissions,
}
```

```rust
struct CheckResult {
    id: String,
    category: Category,
    status: Status,
    message: String,
    recommendation: Option<String>,
}
```

Crear también:

```rust
struct AuditReport {
    checks: Vec<CheckResult>,
    summary: Summary,
}
```

### Criterio de finalización

Todos los checks producen `Vec<CheckResult>` y ninguna regla imprime directamente.

---

## v0.5 — Auditor central

### Objetivo

Crear el orquestador que ejecuta todos los checks.

### Funcionalidades

- Definir rutas configurables.
- Ejecutar usuarios.
- Ejecutar SSH.
- Ejecutar permisos.
- Combinar resultados.
- Calcular resumen.
- Continuar después de errores parciales.

### Rutas por defecto

```text
/etc/passwd
/etc/shadow
/etc/group
/etc/ssh/sshd_config
```

### Rutas para tests

El diseño debe permitir sustituir las rutas del sistema por rutas temporales.

Ejemplo conceptual:

```rust
struct AuditPaths {
    passwd: PathBuf,
    shadow: PathBuf,
    group: PathBuf,
    sshd_config: PathBuf,
}
```

### Criterio de finalización

Una sola función puede generar un `AuditReport` completo.

---

## v0.6 — CLI

### Objetivo

Crear una interfaz de línea de comandos estable.

### Comandos y opciones

```text
cenda
cenda --help
cenda --version
cenda --json
cenda --verbose
```

### Dependencia

Utilizar `clap` con la feature `derive`.

### Criterio de finalización

La aplicación puede ejecutarse desde la terminal y responde correctamente a las opciones básicas.

---

## v0.7 — Salida humana

### Objetivo

Crear una salida clara para usuarios humanos.

### Requisitos

- Mostrar título.
- Agrupar por categoría.
- Mostrar estado.
- Mostrar mensaje.
- Mostrar recomendación cuando exista.
- Mostrar resumen final.

### Ejemplo

```text
Linux Security Audit
====================

[SSH]
  PASS  SSH root login is explicitly disabled
  WARN  SSH password authentication is enabled
      Recommendation: Consider disabling password authentication

Summary
-------
PASS: 1
WARN: 1
INFO: 0
ERROR: 0
```

### Criterio de finalización

La salida humana es legible y no contiene lógica específica de auditoría.

---

## v0.8 — Salida JSON

### Objetivo

Permitir que otras herramientas consuman el resultado.

### Funcionalidades

- Serializar `AuditReport`.
- Incluir `schema_version`.
- Incluir información de la herramienta.
- Incluir todos los checks.
- Incluir resumen.
- Mantener nombres de campos estables.

### Criterio de finalización

El resultado de:

```bash
cenda --json
```

es JSON válido y puede ser procesado por herramientas externas.

---

## v0.9 — Códigos de salida

### Objetivo

Preparar la herramienta para automatización.

### Reglas

```text
0 = No hay warnings
1 = Existe al menos un warning
2 = Error fatal de ejecución
```

### Consideraciones

- Los resultados `INFO` no deben causar código `1`.
- Los errores parciales deberían aparecer en el reporte.
- Solo los errores que impidan producir un reporte útil deberían causar código `2`.

### Criterio de finalización

El código de salida es consistente en modo humano y JSON.

---

## v1.0 — Primera versión estable

### Alcance final

#### Usuarios

- Parseo de `/etc/passwd`.
- Detección de entradas malformadas.
- Detección de cuentas adicionales con UID `0`.
- Conteo de usuarios.
- Conteo de shells interactivas.

#### SSH

- Detección explícita de `PermitRootLogin`.
- Detección explícita de `PasswordAuthentication`.
- Estado `INFO` cuando la configuración no puede determinarse.
- Limitación documentada respecto a `Include`.

#### Permisos

- Revisión de `/etc/passwd`.
- Revisión de `/etc/group`.
- Revisión de `/etc/shadow`.
- Inspección de bits de lectura y escritura relevantes.

#### CLI

- `--help`.
- `--version`.
- `--json`.
- `--verbose`.

#### Calidad

- Tests unitarios.
- Tests de parsing.
- Tests de reglas.
- Tests de serialización.
- Tests de integración de CLI.
- README completo.
- Documentación de limitaciones.
- Código formateado.
- Clippy sin warnings.

### Criterio de finalización

La v1 se considera terminada cuando puede ejecutar:

```bash
cenda
```

y producir un reporte útil, confiable, testeado y documentado.

---

## v1.1 — Mejoras posibles

Después de completar la v1 se podrían considerar:

- Más directivas de SSH.
- Mejor resolución de `Include`.
- Uso opcional de `sshd -T`.
- Estado del firewall.
- Configuración personalizada de checks.
- Archivo de configuración TOML.
- Severidades configurables.
- Exclusión de checks.
- Mejor identificación de shells.
- Información más detallada de ownership.

Estas funcionalidades no forman parte de la primera versión.

---

## v1.2 — Integración con automatización

Posibles mejoras:

- GitHub Actions.
- Docker image.
- Salida SARIF.
- Integración con pipelines.
- Configuración de thresholds.
- Artefactos de auditoría.
- Comparación entre ejecuciones.

---

## v2 — Funcionalidades avanzadas

Solo considerar después de que la v1 sea estable:

- Auditoría de systemd.
- Análisis de servicios.
- Revisión más profunda del firewall.
- Auditoría de contenedores.
- Histórico de resultados.
- Auditoría remota.
- Ejecución periódica.
- Integración con plataformas DevSecOps.

## Regla del roadmap

No agregar una funcionalidad solamente porque es técnicamente posible.

Una nueva característica debe justificar:

- Valor para el usuario.
- Complejidad añadida.
- Impacto en seguridad.
- Impacto en testing.
- Compatibilidad con el alcance del proyecto.