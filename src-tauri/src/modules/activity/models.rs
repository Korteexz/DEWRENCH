use std::collections::BTreeMap;

use serde::Serialize;

/// Um acontecimento datado, de qualquer ferramenta.
///
/// Os campos são deliberadamente genéricos. `kind` e `metadata` carregam o que
/// é específico de cada fonte, de modo que acrescentar uma fonte nova não muda
/// o formato nem quebra quem já consome.
///
/// `timestamp` é epoch em segundos (UTC) e `utc_offset_minutes` guarda o fuso
/// de quem gerou o evento: agrupar por dia exige saber o dia de QUEM fez, não
/// o dia de quem está olhando.
#[derive(Serialize, Debug, Clone)]
pub struct ActivityEvent {
    pub id: String,
    pub timestamp: i64,
    pub utc_offset_minutes: i32,
    /// De onde o evento veio: `git` hoje; `docker`, `ci`, `agent` depois.
    pub source: String,
    /// Máquina de origem. Nulo quando local — colaboração ainda não existe.
    pub machine: Option<String>,
    pub actor: Option<String>,
    /// Módulo do DEWRENCH ao qual o evento pertence.
    pub module: String,
    /// Tipo dentro da fonte: `commit`, `merge`, `revert`, `root`…
    pub kind: String,
    pub repository: String,
    pub branch: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

/// Resposta de uma coleta, com as limitações declaradas.
#[derive(Serialize, Debug)]
pub struct ActivityStream {
    pub events: Vec<ActivityEvent>,
    /// Fontes que responderam. Uma fonte ausente não é erro.
    pub sources: Vec<String>,
    /// A coleta bateu no teto e há mais eventos além dos devolvidos.
    pub truncated: bool,
}
