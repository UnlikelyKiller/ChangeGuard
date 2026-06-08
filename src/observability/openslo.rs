use serde::{Deserialize, Serialize};
use crate::state::graph_kinds::NodeKind;
use crate::platform::urn::build_urn;
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloFile {
    pub api_version: String,
    pub kind: String,
    pub metadata: OpenSloMetadata,
    pub spec: OpenSloSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloMetadata {
    pub name: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloSpec {
    pub service: Option<String>,
    pub indicator: Option<OpenSloIndicator>,
    pub objectives: Option<Vec<OpenSloObjective>>,
    pub alert_policies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloIndicator {
    pub threshold_metric: Option<OpenSloMetric>,
    pub ratio_metric: Option<OpenSloRatioMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloMetric {
    pub metric_source: OpenSloMetricSource,
    pub metric_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloRatioMetric {
    pub good: OpenSloMetric,
    pub total: OpenSloMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloMetricSource {
    pub metric_source_ref: String,
    #[serde(rename = "type")]
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSloObjective {
    pub target: f64,
    pub display_name: Option<String>,
    pub value: Option<f64>,
}

pub struct ParsedSlo {
    pub urn: String,
    pub name: String,
    pub service_name: Option<String>,
    pub metrics: Vec<ParsedMetric>,
    pub alerts: Vec<String>,
    pub metadata: serde_json::Value,
}

pub struct ParsedMetric {
    pub urn: String,
    pub name: String,
    pub query: String,
    pub source: String,
}

pub fn parse_openslo(yaml: &str) -> Result<Vec<ParsedSlo>, String> {
    let docs: Vec<serde_yaml::Value> = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    let mut slos = Vec::new();

    for doc in docs {
        let slo_file: OpenSloFile = serde_yaml::from_value(doc).map_err(|e| e.to_string())?;
        
        if slo_file.kind.to_lowercase() != "slo" {
            continue;
        }

        let mut metrics = Vec::new();
        if let Some(indicator) = &slo_file.spec.indicator {
            if let Some(tm) = &indicator.threshold_metric {
                metrics.push(ParsedMetric {
                    urn: build_urn(NodeKind::Metric, &format!("{}-threshold", slo_file.metadata.name)),
                    name: format!("{}-threshold", slo_file.metadata.name),
                    query: tm.metric_query.clone(),
                    source: tm.metric_source.source_type.clone(),
                });
            }
            if let Some(rm) = &indicator.ratio_metric {
                metrics.push(ParsedMetric {
                    urn: build_urn(NodeKind::Metric, &format!("{}-good", slo_file.metadata.name)),
                    name: format!("{}-good", slo_file.metadata.name),
                    query: rm.good.metric_query.clone(),
                    source: rm.good.metric_source.source_type.clone(),
                });
                metrics.push(ParsedMetric {
                    urn: build_urn(NodeKind::Metric, &format!("{}-total", slo_file.metadata.name)),
                    name: format!("{}-total", slo_file.metadata.name),
                    query: rm.total.metric_query.clone(),
                    source: rm.total.metric_source.source_type.clone(),
                });
            }
        }

        let slo_urn = build_urn(NodeKind::Slo, &slo_file.metadata.name);
        slos.push(ParsedSlo {
            urn: slo_urn,
            name: slo_file.metadata.name.clone(),
            service_name: slo_file.spec.service.clone(),
            metrics,
            alerts: slo_file.spec.alert_policies.unwrap_or_default(),
            metadata: json!({
                "displayName": slo_file.metadata.display_name,
                "apiVersion": slo_file.api_version,
            }),
        });
    }

    Ok(slos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_slo() {
        let yaml = r#"
apiVersion: openslo/v1
kind: SLO
metadata:
  name: user-service-availability
  displayName: User Service Availability
spec:
  service: user-service
  indicator:
    thresholdMetric:
      metricSource:
        metricSourceRef: prometheus
        type: prometheus
      metricQuery: sum(rate(http_requests_total{status=~"2.."}[5m]))
  objectives:
    - target: 0.999
  alertPolicies:
    - high-error-rate
"#;
        let slos = parse_openslo(yaml).unwrap();
        assert_eq!(slos.len(), 1);
        let slo = &slos[0];
        assert_eq!(slo.name, "user-service-availability");
        assert_eq!(slo.service_name, Some("user-service".to_string()));
        assert_eq!(slo.metrics.len(), 1);
        assert_eq!(slo.metrics[0].source, "prometheus");
        assert_eq!(slo.alerts.len(), 1);
        assert_eq!(slo.alerts[0], "high-error-rate");
    }

    #[test]
    fn test_parse_ratio_slo() {
        let yaml = r#"
apiVersion: openslo/v1
kind: SLO
metadata:
  name: user-service-reliability
spec:
  service: user-service
  indicator:
    ratioMetric:
      good:
        metricSource:
          metricSourceRef: prom
          type: prometheus
        metricQuery: http_requests_total{status=~"2.."}
      total:
        metricSource:
          metricSourceRef: prom
          type: prometheus
        metricQuery: http_requests_total{}
  objectives:
    - target: 0.99
"#;
        let slos = parse_openslo(yaml).unwrap();
        assert_eq!(slos.len(), 1);
        let slo = &slos[0];
        assert_eq!(slo.metrics.len(), 2);
    }
}
