# Diseño técnico de Cenda

## 1. Objetivo

`cenda` es una CLI de solo lectura escrita en Rust para revisar configuraciones básicas de seguridad de Linux.

El diseño prioriza:

- Simplicidad.
- Separación de responsabilidades.
- Testabilidad.
- Manejo explícito de errores.
- Resultados estructurados.
- Independencia entre la lógica de auditoría y la presentación.

La herramienta no intenta resolver toda la semántica de seguridad de Linux. Cada check debe tener un alcance pequeño, documentado y verificable.

## 2. Arquitectura general

La aplicación seguirá este flujo:

```text
Sistema operativo
        |
        v
Lectura de archivos y metadata
        |
        v
Parsers
        |
        v
Reglas de auditoría
        |
        v
CheckResult
        |
        +------------------+
        |                  |
        v                  v
Salida humana          Salida JSON
```

La lógica de seguridad no debería imprimir directamente en pantalla ni construir JSON.

Los checks deben producir datos estructurados. La capa de salida decide cómo representar esos datos.

## 3. Estructura propuesta

```text
cenda/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── design.md
├── roadmap.md
├── security-model.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── model.rs
│   ├── audit.rs
│   ├── output.rs
│   └── checks/
│       ├── mod.rs
│       ├── passwd.rs
│       ├── ssh.rs
│       └── permissions.rs
└── tests/
    ├── cli.rs
    ├── passwd.rs
    ├── ssh.rs
    └── permissions.rs
```

La estructura puede comenzar siendo más pequeña. No es necesario crear todos los módulos antes de necesitarlos.

## 4. Módulos

### `main.rs`

Responsabilidades:

- Inicializar la aplicación.
- Interpretar el resultado final.
- Convertir el estado de la auditoría en un código de salida.
- Evitar contener lógica específica de seguridad.

Ejemplo conceptual:

```rust
fn main() {
    let cli = cli::parse();

    match audit::run(&cli) {
        Ok(report) => {
            output::render(&report, cli.output_format);
            std::process::exit(report.exit_code());
        }
        Err(error) => {
            eprintln!("ERROR: {error}");
            std::process::exit(2);
        }
    }
}
```

### `cli.rs`

Responsabilidades:

- Definir los argumentos de línea de comandos.
- Exponer `--help`.
- Exponer `--version`.
- Seleccionar salida humana o JSON.
- Activar el modo verbose.

La CLI no debería ejecutar directamente las reglas de seguridad.

### `model.rs`

Contendrá los tipos compartidos por todos los módulos.

Modelo conceptual:

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

```rust
struct Summary {
    pass: usize,
    warn: usize,
    info: usize,
    error: usize,
}
```

```rust
struct AuditReport {
    tool: String,
    version: String,
    schema_version: u32,
    checks: Vec<CheckResult>,
    summary: Summary,
}
```

Los tipos destinados a JSON deberían derivar `Serialize` mediante Serde.

### `audit.rs`

Responsabilidades:

- Ejecutar los checks.
- Coordinar las rutas del sistema.
- Combinar los resultados.
- Crear el `AuditReport`.
- Determinar si hubo warnings o errores.

Ejemplo conceptual:

```rust
pub struct AuditPaths {
    pub passwd: PathBuf,
    pub shadow: PathBuf,
    pub group: PathBuf,
    pub sshd_config: PathBuf,
}
```

En producción, estas rutas apuntarían a:

```text
/etc/passwd
/etc/shadow
/etc/group
/etc/ssh/sshd_config
```

En tests, podrían apuntar a archivos temporales.

### `checks/passwd.rs`

Responsabilidades:

- Leer y parsear `/etc/passwd`.
- Detectar entradas malformadas.
- Detectar cuentas adicionales con UID `0`.
- Contar usuarios.
- Identificar shells potencialmente interactivas.

Debe separar el parser de la lectura del archivo:

```rust
pub fn parse_passwd(input: &str) -> Result<PasswdReport, ParseError>
```

Y la ejecución del check:

```rust
pub fn audit_passwd_file(path: &Path) -> Vec<CheckResult>
```

El parser no debería conocer `/etc/passwd`.

### `checks/ssh.rs`

Responsabilidades:

- Leer `sshd_config`.
- Ignorar comentarios.
- Tolerar espacios y tabulaciones.
- Detectar directivas explícitas.
- Informar cuando un valor no puede determinarse.
- Documentar limitaciones relacionadas con `Include`.

Funciones conceptuales:

```rust
pub fn parse_sshd_config(input: &str) -> SshConfig
```

```rust
pub fn audit_sshd_config(path: &Path) -> Vec<CheckResult>
```

La primera versión no necesita resolver completamente todos los defaults ni todos los includes de OpenSSH.

### `checks/permissions.rs`

Responsabilidades:

- Leer metadata de archivos.
- Inspeccionar owner, group y mode bits.
- Aplicar reglas específicas por archivo.
- No modificar ningún permiso.

Funciones conceptuales:

```rust
pub fn inspect_permissions(path: &Path) -> Result<FilePermissions, PermissionError>
```

```rust
pub fn audit_critical_file(path: &Path, policy: FilePolicy) -> CheckResult
```

Las reglas deben ser explícitas. No se debe asumir que cualquier modo distinto de `0600` es inseguro.

### `output.rs`

Responsabilidades:

- Renderizar un `AuditReport` para humanos.
- Serializar un `AuditReport` como JSON.
- Mantener los formatos separados de la lógica de auditoría.

Funciones conceptuales:

```rust
pub fn render_human(report: &AuditReport)
```

```rust
pub fn render_json(report: &AuditReport) -> Result<String, serde_json::Error>
```

## 5. Modelo de estado

La versión inicial utilizará cuatro estados:

```text
PASS
WARN
INFO
ERROR
```

### `PASS`

La comprobación observada cumple la regla definida.

Ejemplo:

```text
PASS  /etc/shadow is not world-readable
```

### `WARN`

Se detectó una condición potencialmente insegura.

Ejemplo:

```text
WARN  SSH password authentication is enabled
```

### `INFO`

La herramienta pudo ejecutar el check, pero no dispone de suficiente información para afirmar que la configuración es segura o insegura.

Ejemplo:

```text
INFO  Could not determine effective PermitRootLogin setting
```

### `ERROR`

No fue posible ejecutar una comprobación concreta.

Ejemplo:

```text
ERROR Could not inspect /etc/shadow: permission denied
```

Un error individual no necesariamente debe terminar la auditoría completa.

## 6. Códigos de salida

```text
0 = No se encontraron warnings
1 = Se encontraron uno o más warnings
2 = Error fatal de ejecución
```

Un resultado `INFO` no debe producir automáticamente código de salida `1`.

Los errores parciales deben documentarse dentro del reporte. El código `2` se reserva para situaciones en las que el programa no puede producir un reporte útil.

## 7. Dependencias

Dependencias iniciales recomendadas:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### `clap`

Para definir la interfaz de línea de comandos.

### `serde`

Para derivar serialización de las estructuras del reporte.

### `serde_json`

Para generar la salida JSON.

### `nix`

No es obligatorio en la primera versión. La biblioteca estándar de Rust puede ser suficiente para consultar metadata Unix:

```rust
use std::os::unix::fs::{MetadataExt, PermissionsExt};
```

Se incorporará `nix` únicamente si aporta una mejora clara.

### `anyhow`

Puede simplificar errores de aplicación, pero inicialmente se recomienda comprender `Result`, `Option` y errores propios de Rust antes de ocultar demasiados detalles.

## 8. Acceso al sistema

La capa de sistema debe estar separada de la lógica pura.

Ejemplo:

```text
read_file(path)
      |
      v
parse_passwd(content)
      |
      v
evaluate_passwd_rules(report)
      |
      v
Vec<CheckResult>
```

Esto permite probar:

- Archivos inexistentes.
- Archivos vacíos.
- Permisos insuficientes.
- Datos malformados.
- Configuraciones incompletas.

Sin necesidad de modificar el sistema real.

## 9. Parsing de `/etc/passwd`

Una línea válida debe contener siete campos separados por `:`:

```text
username:password:uid:gid:gecos:home:shell
```

El parser debería comprobar:

- Exactamente siete campos.
- Nombre de usuario no vacío.
- UID numérico.
- GID numérico.
- Línea no vacía.
- Línea no malformada.

La contraseña no debe mostrarse ni registrarse.

El valor del campo de contraseña puede ignorarse después de comprobar la estructura.

## 10. Parsing de `sshd_config`

El parser inicial debe:

- Ignorar líneas vacías.
- Ignorar comentarios.
- Aceptar espacios múltiples.
- Aceptar tabulaciones.
- Detectar las directivas relevantes.
- Mantener información sobre valores no reconocidos.
- No asumir una configuración efectiva cuando no puede determinarse.

Directivas iniciales:

```text
PermitRootLogin
PasswordAuthentication
```

Valores relevantes para `PermitRootLogin`:

```text
yes
no
prohibit-password
without-password
```

La primera versión puede limitarse a revisar el archivo principal y documentar que no resuelve totalmente las directivas `Include`.

## 11. Inspección de permisos

La revisión utilizará metadata Unix.

Para `/etc/shadow`, la regla inicial será:

```text
WARN si el archivo es legible por others
PASS en caso contrario
```

Para `/etc/passwd` y `/etc/group`:

```text
WARN si group u others tienen permiso de escritura
PASS en caso contrario
```

La versión inicial no pretende analizar:

- ACLs.
- SELinux contexts.
- AppArmor.
- Atributos extendidos.
- Montajes especiales.
- Ownership complejo.
- Políticas específicas de cada distribución.

## 12. Testabilidad

Los parsers deben ser funciones puras siempre que sea posible.

Ejemplo:

```rust
#[test]
fn detects_additional_uid_zero_accounts() {
    let input = "\
root:x:0:0:root:/root:/bin/bash
backup:x:0:0:backup:/home/backup:/bin/bash
";

    let report = parse_passwd(input).unwrap();

    assert_eq!(report.uid_zero_accounts.len(), 2);
}
```

Los tests de filesystem pueden utilizar:

- Directorios temporales.
- Archivos temporales.
- Rutas inyectables.
- Metadata controlada cuando el sistema de tests lo permita.

## 13. Principios de diseño

### Solo lectura

La herramienta observa y reporta. No corrige.

### Resultados estructurados

La lógica produce `CheckResult`. No imprime directamente.

### Errores explícitos

La ausencia de información no debe convertirse en una conclusión inventada.

### Reglas pequeñas

Cada check debe representar una sola decisión comprensible.

### Limitaciones documentadas

Si una comprobación no cubre toda la semántica del sistema, debe indicarse en la documentación.

### Dependencias mínimas

No incorporar una biblioteca si la funcionalidad puede implementarse de forma clara con la biblioteca estándar.

### Compatibilidad con automatización

El JSON y los códigos de salida deben ser estables y predecibles.