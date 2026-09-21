# Modelo de seguridad de cenda

## 1. Propósito

Este documento define qué considera `cenda` una condición correcta, una advertencia, un estado informativo o un error.

El objetivo es evitar interpretaciones ambiguas y dejar claro qué significa cada resultado.

`cenda` es un auditor de configuración básica. No es un sistema completo de evaluación de seguridad.

## 2. Principio principal

Un resultado `PASS` solo significa:

> La comprobación específica ejecutada cumplió la regla implementada.

No significa:

- Que el sistema sea completamente seguro.
- Que no existan vulnerabilidades.
- Que no haya malware.
- Que la configuración efectiva completa sea segura.
- Que los servicios expuestos sean seguros.
- Que los usuarios estén correctamente autorizados.
- Que no existan problemas fuera del alcance del check.

## 3. Estados

## `PASS`

La configuración observada cumple la regla.

Ejemplo:

```text
PASS  /etc/shadow is not world-readable
```

## `WARN`

Se detectó una condición que merece revisión.

Ejemplo:

```text
WARN  SSH password authentication is enabled
```

Un warning no siempre significa que exista una vulnerabilidad explotable. Puede representar una configuración que depende de la política de seguridad del sistema.

## `INFO`

La herramienta pudo ejecutar la comprobación, pero no puede concluir que la configuración sea segura o insegura.

Ejemplo:

```text
INFO  Could not determine effective PermitRootLogin setting
```

Este estado es especialmente importante cuando falta información o la configuración tiene semántica que la primera versión no resuelve completamente.

## `ERROR`

No fue posible ejecutar una comprobación.

Ejemplo:

```text
ERROR Could not inspect /etc/shadow: permission denied
```

Los errores deben incluir una explicación breve y permitir continuar con otros checks cuando sea posible.

## 4. Código de salida

```text
0 = No se detectaron warnings
1 = Se detectó al menos un warning
2 = Error fatal de ejecución
```

Los resultados `INFO` no producen código `1`.

Los errores parciales tampoco deberían producir automáticamente código `2` si la herramienta puede generar un reporte útil.

## 5. Modelo de amenaza

`cenda` considera principalmente errores básicos de configuración local:

- Cuentas adicionales con UID `0`.
- Entradas corruptas o malformadas en `/etc/passwd`.
- Archivos críticos escribibles por usuarios no privilegiados.
- `/etc/shadow` legible por otros usuarios.
- Autenticación SSH mediante contraseña habilitada.
- Login SSH de root explícitamente habilitado.

No intenta detectar atacantes, malware, explotación activa ni compromiso del sistema.

## 6. Modelo de confianza

La herramienta asume que:

- El binario no fue reemplazado.
- El sistema operativo proporciona metadata confiable.
- Los archivos leídos pertenecen al sistema que se está auditando.
- El usuario tiene permisos suficientes para realizar las comprobaciones.
- El entorno no está siendo manipulado durante la ejecución.
- No existen mecanismos externos que alteren la interpretación normal de los archivos.

Si alguna de estas suposiciones no se cumple, el resultado puede ser incompleto o incorrecto.

## 7. Usuarios y `/etc/passwd`

## 7.1 Estructura esperada

Una entrada normal contiene siete campos separados por `:`:

```text
username:password:uid:gid:gecos:home:shell
```

El parser considera potencialmente malformada una línea si:

- No contiene siete campos.
- El nombre de usuario está vacío.
- El UID no es numérico.
- El GID no es numérico.
- La estructura no puede interpretarse correctamente.

Ejemplo:

```text
WARN  Malformed /etc/passwd entry at line 12
```

## 7.2 UID `0`

Un UID `0` tiene privilegios equivalentes al usuario root en muchos contextos del sistema.

La herramienta revisa cuentas con:

```text
UID = 0
```

El resultado esperado es:

```text
PASS  No additional UID 0 accounts detected
```

Si existe otra cuenta además de `root`:

```text
WARN  Additional UID 0 account detected: backup
```

La herramienta no determina automáticamente si esa cuenta es legítima. Solo indica que debe revisarse.

## 7.3 Shells interactivas

La herramienta puede identificar shells potencialmente interactivas mediante una heurística.

Ejemplos de shells interactivas conocidas:

```text
/bin/bash
/bin/sh
/bin/zsh
/bin/fish
/bin/ksh
```

Ejemplos de shells normalmente no interactivas:

```text
/bin/false
/usr/bin/false
/sbin/nologin
/usr/sbin/nologin
```

El resultado se presenta como información:

```text
INFO  Interactive shell users: 2
```

No se interpreta automáticamente como una vulnerabilidad.

Esta lógica puede producir falsos positivos o falsos negativos porque un sistema puede utilizar shells válidas en rutas diferentes.

## 8. SSH

## 8.1 Alcance

La versión inicial analiza explícitamente:

```text
/etc/ssh/sshd_config
```

Directivas principales:

```text
PermitRootLogin
PasswordAuthentication
```

## 8.2 `PermitRootLogin`

Resultados posibles:

```text
PermitRootLogin no
```

Resultado:

```text
PASS  SSH root login is explicitly disabled
```

```text
PermitRootLogin yes
```

Resultado:

```text
WARN  SSH root login is explicitly enabled
```

```text
PermitRootLogin prohibit-password
```

Resultado recomendado:

```text
INFO  SSH root login allows non-password authentication
```

```text
PermitRootLogin without-password
```

Resultado recomendado:

```text
INFO  SSH root login allows non-password authentication
```

Si no existe una directiva explícita:

```text
INFO  Could not determine effective PermitRootLogin setting
```

La ausencia de la directiva no debe interpretarse automáticamente como `yes` o `no`.

## 8.3 `PasswordAuthentication`

```text
PasswordAuthentication no
```

Resultado:

```text
PASS  SSH password authentication is disabled
```

```text
PasswordAuthentication yes
```

Resultado:

```text
WARN  SSH password authentication is enabled
```

Si no se puede determinar:

```text
INFO  Could not determine effective PasswordAuthentication setting
```

## 8.4 Comentarios y formato

El parser debe tolerar:

```text
PermitRootLogin no
PermitRootLogin    no
PermitRootLogin<TAB>no
```

También debe ignorar:

```text
# PermitRootLogin yes
```

Las directivas no reconocidas deben conservarse o ignorarse de forma segura, pero no deben producir conclusiones inventadas.

## 8.5 Directivas repetidas

OpenSSH puede procesar directivas repetidas según reglas específicas.

La primera versión debe documentar su política. Una opción sencilla es:

- Registrar todas las apariciones.
- Usar la última aparición visible del archivo.
- Marcar como `INFO` los casos ambiguos.
- No afirmar que se conoce la configuración efectiva si existen `Include` no resueltos.

La política exacta debe estar cubierta por tests.

## 8.6 `Include`

OpenSSH puede incluir otros archivos:

```text
Include /etc/ssh/sshd_config.d/*.conf
```

La primera versión puede no resolver automáticamente esos archivos.

Si se detecta una directiva `Include`, la herramienta no debería afirmar con demasiada seguridad que conoce la configuración efectiva.

Posibles resultados:

```text
INFO  SSH configuration contains Include directives that are not fully resolved
```

La limitación debe aparecer en la documentación.

## 9. Permisos de archivos

## 9.1 `/etc/shadow`

Regla inicial:

```text
WARN si /etc/shadow es legible por others
PASS en caso contrario
```

Resultado correcto:

```text
PASS  /etc/shadow is not world-readable
```

Resultado de advertencia:

```text
WARN  /etc/shadow is world-readable
```

No se debe asumir que cualquier permiso de lectura para el grupo `shadow` es automáticamente inseguro, porque algunas distribuciones utilizan este grupo de forma intencional.

La versión inicial tampoco pretende validar completamente:

- Ownership.
- ACLs.
- SELinux.
- AppArmor.
- Permisos especiales.
- Políticas específicas de cada distribución.

## 9.2 `/etc/passwd`

`/etc/passwd` normalmente debe ser legible por usuarios del sistema.

La regla inicial no debe marcar como warning que el archivo sea world-readable.

Regla propuesta:

```text
WARN si group u others tienen permiso de escritura
PASS en caso contrario
```

Resultados:

```text
PASS  /etc/passwd is not writable by group or other users
```

```text
WARN  /etc/passwd is writable by unauthorized users
```

Esta es una política simplificada. La herramienta no determina por sí sola quién está autorizado a modificar el archivo.

## 9.3 `/etc/group`

La regla será equivalente a `/etc/passwd`:

```text
WARN si group u others tienen permiso de escritura
PASS en caso contrario
```

Resultado esperado:

```text
PASS  /etc/group is not writable by group or other users
```

## 9.4 Ownership

La herramienta puede mostrar o registrar:

```text
Owner: root
Group: root
Mode: 0644
```

Pero la versión inicial no debe intentar modelar todos los casos de ownership y autorización posibles.

## 10. Archivos ausentes

La ausencia de un archivo no debe interpretarse automáticamente como una vulnerabilidad.

Ejemplos:

```text
INFO  SSH configuration file was not found
```

```text
INFO  /etc/shadow was not found
```

El resultado exacto puede variar según el archivo, pero el principio general es:

> No inventar una conclusión cuando no existe información suficiente.

## 11. Errores de permisos

Si el usuario no puede leer un archivo:

```text
WARN  Could not inspect /etc/shadow: permission denied
```

La herramienta debe:

- Explicar el problema.
- Continuar con otros checks.
- Evitar imprimir información sensible.
- No intentar elevar privilegios automáticamente.
- No ejecutar `sudo`.
- No modificar permisos.

## 12. Información sensible

La herramienta debe evitar imprimir:

- Contenido completo de `/etc/shadow`.
- Hashes de contraseñas.
- Valores sensibles de archivos.
- Información innecesaria de cuentas.
- Tokens o credenciales.

El reporte solo necesita mostrar metadatos y conclusiones mínimas.

Por ejemplo, es aceptable mostrar:

```text
WARN  Additional UID 0 account detected: backup
```

Pero no es necesario mostrar la línea completa de `/etc/passwd`.

## 13. Solo lectura

`cenda` nunca debe:

- Modificar `/etc/passwd`.
- Modificar `/etc/shadow`.
- Modificar `/etc/group`.
- Modificar `sshd_config`.
- Cambiar permisos.
- Crear usuarios.
- Eliminar usuarios.
- Instalar paquetes.
- Abrir puertos.
- Cerrar puertos.
- Reiniciar servicios.
- Aplicar recomendaciones automáticamente.

Las recomendaciones son únicamente informativas.

## 14. Limitaciones generales

`cenda` no es:

- Un antivirus.
- Un SIEM.
- Un sistema de detección de intrusiones.
- Un vulnerability scanner.
- Un pentest automatizado.
- Un escáner de red.
- Un sistema de cumplimiento normativo completo.
- Una herramienta de remediación.
- Una garantía de seguridad del sistema.

El resultado debe interpretarse dentro del alcance de los checks implementados.

## 15. Principio de comunicación responsable

La herramienta debe preferir un resultado incompleto pero honesto antes que una conclusión falsa.

Ejemplo correcto:

```text
INFO  Could not determine effective PasswordAuthentication setting
```

Ejemplo incorrecto:

```text
PASS  SSH password authentication is disabled
```

cuando la directiva no fue encontrada y la configuración efectiva no puede calcularse.

## 16. Evolución futura del modelo

En versiones posteriores podrían incorporarse:

- Severidades configurables.
- Políticas por distribución.
- Resolución completa de `Include`.
- Uso opcional de `sshd -T`.
- Reglas personalizadas.
- Configuración TOML o YAML.
- Salida SARIF.
- Integración con CI/CD.
- Excepciones documentadas.
- Cumplimiento con perfiles de seguridad.

Estas funcionalidades deben añadirse sin debilitar el principio central:

> La herramienta debe reportar únicamente conclusiones respaldadas por la información que realmente pudo observar.