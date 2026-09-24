/// Resultado de una comprobación individual.
///
/// Las variantes representan estados observables del check; `Info` no implica
/// que la configuración sea segura o insegura, sino que falta información.
pub enum Status {
    Pass,
    Warn,
    Info,
    Error,
}

/// Área del sistema a la que pertenece una comprobación.
pub enum Category {
    Ssh,
    Users,
    Permissions,
}

/// Datos estructurados producidos por un check.
///
/// Este tipo no imprime ni decide cómo mostrar el resultado. La capa de salida
/// podrá usar estos datos para generar texto humano o JSON.
pub struct CheckResult {
    /// Identificador estable para distinguir este check de otros.
    pub id: String,
    /// Área funcional del check.
    pub category: Category,
    /// Estado obtenido al aplicar la regla.
    pub status: Status,
    /// Explicación breve del resultado.
    pub message: String,
    /// Recomendación opcional; algunos resultados no necesitan una.
    pub recommendation: Option<String>,
}
