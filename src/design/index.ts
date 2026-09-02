/**
 * Superfície pública do design system do DEWRENCH.
 *
 * Regra de dependência: `design/` não importa nada de `app/` nem de
 * `modules/`. Os módulos importam daqui — nunca o contrário. Isso é o que
 * permite adicionar Docker, Kubernetes ou Terraform depois sem reescrever a
 * linguagem visual.
 */
import './tokens/index.css'
import './primitives/primitives.css'

export { Panel } from './primitives/Panel'
export { SectionHeader } from './primitives/SectionHeader'
export { TechnicalLabel } from './primitives/TechnicalLabel'
export { Divider } from './primitives/Divider'
export { Metric, MetricCluster } from './primitives/Metric'
export { StatusIndicator } from './primitives/StatusIndicator'
export { TelemetryBar } from './primitives/TelemetryBar'
export { DataRow } from './primitives/DataRow'
export { CoordinateLabel } from './primitives/CoordinateLabel'
export { InstrumentFrame } from './primitives/InstrumentFrame'
export { Button } from './primitives/Button'

export type { LabelTone } from './primitives/TechnicalLabel'
export type { MetricTone } from './primitives/Metric'
export type { StatusTone } from './primitives/StatusIndicator'
